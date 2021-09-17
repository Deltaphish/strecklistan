use crate::app::Msg;
use crate::generated::css_classes::C;
use crate::page::loading::Loading;
use crate::util::{simple_ev, DATE_INPUT_FMT};
use chrono::{DateTime, Datelike, Duration, IsoWeek, NaiveDate, Utc, Weekday};
use seed::app::cmds::timeout;
use seed::{prelude::*, *};
use seed_fetcher::Resources;
use seed_fetcher::{event, NotAvailable, ResourceStore};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use strecklistan_api::{
    inventory::{InventoryItemId, InventoryItemStock},
    transaction::Transaction,
};

#[derive(Clone, Debug)]
pub enum AnalyticsMsg {
    ComputeCharts,
    ChartsComputed((Rc<HashMap<InventoryItemId, Node<AnalyticsMsg>>>, Rc<Vec<(String,i32)>>)),
    SetStartDate(String),
    SetEndDate(String),

    // -- Resource Events -- //
    ResFetched(event::Fetched),
    ResMarkDirty(event::MarkDirty),
}

//#[derive(Clone)]
pub struct AnalyticsPage {
    /// An index of the inventory stocks at the start of every week
    //inventory_by_week: Rc<BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>>>,

    /// Pre-computed and cached charts
    charts: Rc<HashMap<InventoryItemId, Node<AnalyticsMsg>>>,

    top10: Rc<Vec<(String,i32)>>,

    /// Handle for the process computing the charts
    charts_job: Option<CmdHandle>,

    /// Start-date filter for computing charts
    start_date: DateTime<Utc>,

    /// End-date filter for computing charts
    end_date: DateTime<Utc>,
}

#[derive(Resources)]
struct Res<'a> {
    #[url = "/api/transactions"]
    transactions: &'a Vec<Transaction>,

    #[url = "/api/inventory/items"]
    inventory: &'a HashMap<InventoryItemId, InventoryItemStock>,
}

impl AnalyticsPage {
    pub fn new(rs: &ResourceStore, orders: &mut impl Orders<AnalyticsMsg>) -> Self {
        orders.subscribe(AnalyticsMsg::ResFetched);
        orders.subscribe(AnalyticsMsg::ResMarkDirty);
        Res::acquire(rs, orders).ok();

        let now = Utc::now();
        AnalyticsPage {
            charts: Rc::new(HashMap::new()),
            top10: Rc::new(Vec::new()),
            charts_job: None,
            start_date: now - Duration::days(365),
            end_date: now,
        }
    }

    pub fn update(
        &mut self,
        msg: AnalyticsMsg,
        rs: &ResourceStore,
        orders: &mut impl Orders<Msg>,
    ) -> Result<(), NotAvailable> {
        let res = Res::acquire(rs, orders)?;

        let mut orders_local = orders.proxy(|msg| Msg::AnalyticsMsg(msg));

        match msg {
            AnalyticsMsg::ComputeCharts => {
                self.compute_charts(&res, &mut orders_local);
            }
            AnalyticsMsg::ChartsComputed((charts,top10)) => {
                self.charts = charts;
                self.top10 = top10;
                self.charts_job = None;
            }
            AnalyticsMsg::SetStartDate(input) => {
                if let Ok(date) = NaiveDate::parse_from_str(&input, DATE_INPUT_FMT) {
                    self.start_date = DateTime::from_utc(date.and_hms(0, 0, 0), Utc);
                }
            }
            AnalyticsMsg::SetEndDate(input) => {
                if let Ok(date) = NaiveDate::parse_from_str(&input, DATE_INPUT_FMT) {
                    self.end_date = DateTime::from_utc(date.and_hms(0, 0, 0), Utc);
                }
            }

            AnalyticsMsg::ResFetched(_) => {}
            AnalyticsMsg::ResMarkDirty(_) => {}
        }

        Ok(())
    }

    pub fn view(&self, rs: &ResourceStore) -> Node<Msg> {
        let _res = match Res::acquire_now(rs) {
            Ok(res) => res,
            Err(_) => return Loading::view(),
        };

        if self.charts_job.is_some() {
            return div![
                C![C.accounting_page],
                h2!["Laddar statistik..."],
                Loading::view(),
            ];
        }

        div![
            C![C.accounting_page],
            div![
                input![
                    attrs! {At::Type => "date"},
                    attrs! {At::Value => self.start_date.format(DATE_INPUT_FMT).to_string()},
                    input_ev(Ev::Input, |input| AnalyticsMsg::SetStartDate(input)),
                ],
                input![
                    attrs! {At::Type => "date"},
                    attrs! {At::Value => self.end_date.format(DATE_INPUT_FMT).to_string()},
                    input_ev(Ev::Input, |input| AnalyticsMsg::SetEndDate(input)),
                ],
                if self.charts_job.is_some() {
                    button![
                        C![C.wide_button],
                        div![
                            C![C.lds_ripple],
                            style! {
                                St::Position => "absolute",
                                St::MarginTop => "-20px",
                            },
                            div![],
                            div![],
                        ],
                        attrs! { At::Disabled => true },
                        "Beräkna Statistik",
                    ]
                } else {
                    button![
                        C![C.wide_button],
                        simple_ev(Ev::Click, AnalyticsMsg::ComputeCharts),
                        "Beräkna Statistik",
                    ]
                },
            ],
            div![self.top10.iter().map(|(n,t)| div![p![n],p![t]])],
            div![self.charts.values().map(|chart| chart.clone())],
        ]
        .map_msg(|msg| Msg::AnalyticsMsg(msg))
    }

    fn compute_charts(&mut self, res: &Res, orders: &mut impl Orders<AnalyticsMsg>) {
        if self.charts_job.is_some() {
            return;
        }

        self.charts = Rc::new(HashMap::new());

        let inventory_by_week = calculate_inventory_by_week(&res.transactions);
        let inventory = res.inventory.clone();
        let transactions = res.transactions.clone();
        let start_date = self.start_date;
        let end_date = self.end_date;

        self.charts_job = Some(orders.perform_cmd_with_handle(async move {
            // Create Top 10

            let mut counter : HashMap<i32,i32> = HashMap::new();
            for transaction in transactions {
                if &transaction.time > &start_date && &transaction.time < &end_date {
                    for item in transaction.bundles {
                        if item.change < 0 {
                            let item_id = *(item.item_ids.keys().next().unwrap_or(&0));
                            counter.insert(item_id, counter.get(&item_id).unwrap_or(&0) + -1 * (item.change));
                        }
                    }
                }
            }

            let mut top10 : Vec<(String,i32)> = Vec::new();

            for (k,v) in counter {
                let name = inventory.get(&k).map(|i| i.name.clone()).unwrap_or(String::new());
                top10.push((name,v));
            }
            top10.sort_by(|a,b| (*b).1.cmp(&a.1));
            top10.resize(if top10.len() < 10 {top10.len()} else {10},(String::new(),0));

            // Create Weekly overview
            let mut charts = HashMap::new();
            for (id, item) in inventory {
                let chart =
                    plot_sales_over_time(&inventory_by_week, start_date, end_date, id, item.name);

                charts.insert(id, chart);

                timeout(10, || ()).await
            }
            AnalyticsMsg::ChartsComputed((Rc::new(charts), Rc::new(top10)))
        }));
    }
}

fn week_date(week: IsoWeek) -> DateTime<Utc> {
    let naive = NaiveDate::from_isoywd(week.year(), week.week(), Weekday::Mon).and_hms(0, 0, 0);
    DateTime::from_utc(naive, Utc)
}

fn calculate_inventory_by_week(
    transactions_unsorted: &[Transaction],
) -> BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>> {
    let mut transactions = BTreeMap::new();

    for transaction in transactions_unsorted.iter() {
        transactions
            .entry(transaction.time.iso_week())
            .or_insert(vec![])
            .push(transaction.clone());
    }

    let mut inventory = HashMap::new();
    let mut result = BTreeMap::new();

    for (week, weeks_transactions) in transactions.iter() {
        for transaction in weeks_transactions.iter() {
            for bundle in transaction.bundles.iter() {
                for (item_id, item_count) in bundle.item_ids.iter() {
                    *inventory.entry(*item_id).or_default() += *item_count as i32 * bundle.change;
                }
            }
        }
        result.insert(*week, inventory.clone());
    }

    result
}

fn plot_sales_over_time(
    inventory_by_week: &BTreeMap<IsoWeek, HashMap<InventoryItemId, i32>>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    item_id: InventoryItemId,
    name: String,
) -> Node<AnalyticsMsg> {
    let mut iter = inventory_by_week
        .iter()
        .map(|(week, inventory)| {
            let stock = *inventory.get(&item_id).unwrap_or(&0);
            (week_date(*week), stock)
        })
        .filter(|&(date, _)| date >= start_date)
        .filter(|&(date, _)| date <= end_date);

    let mut last_weeks_stock = iter.next().map(|(_, s)| s).unwrap_or(0);
    let points: Vec<(String, u32)> = iter
        .map(|(date, this_weeks_stock)| {
            let sales = last_weeks_stock - this_weeks_stock;
            last_weeks_stock = this_weeks_stock;
            (date, sales)
        })
        .filter(|(_, sales)| *sales >= 0)
        .map(|(date, sales)| {
            let datefmt = format!("{} w{:.02}", date.year(), date.iso_week().week());
            (datefmt, sales as u32)
        })
        .collect();

    plot(name, &points)
}

fn plot<K>(name: String, points: &[(K, u32)]) -> Node<AnalyticsMsg>
where
    K: std::fmt::Display,
{
    let y_max = points.iter().map(|(_, v)| *v).max().unwrap_or_default();
    div![
        h2![name],
        div![
            C![C.chart_histogram],
            points
                .iter()
                .map(|(k, v)| {
                    let percentage = if y_max == 0 { 0 } else { v * 100 / y_max };

                    div![
                        C![C.chart_histogram_col],
                        div![style!(St::FlexBasis => format!("{}%", 100 - percentage)),],
                        div![
                            C![C.chart_histogram_col_line, C.chart_col_tooltip],
                            style!(St::FlexBasis => format!("{}%", percentage)),
                            span![C![C.chart_col_tooltiptext], format!("{}", v),],
                        ],
                        div![C![C.chart_histogram_col_label], format!("{}", k),],
                    ]
                })
                .collect::<Vec<_>>()
        ],
    ]
}

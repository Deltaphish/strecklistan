use schema::tables::event_signups;

/// Metadata about a signed up attendee of an event
#[derive(Queryable, GraphQLObject, Serialize, Deserialize, Debug, PartialEq)]
pub struct Signup {
    pub id: i32,
    pub event: i32,
    pub name: String,
    pub email: String,
}

#[derive(Insertable, GraphQLInputObject, Serialize, Deserialize, Debug, PartialEq)]
#[table_name = "event_signups"]
pub struct NewSignup {
    pub event: i32,
    pub name: String,
    pub email: String,
}

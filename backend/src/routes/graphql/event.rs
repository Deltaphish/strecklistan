use super::context::Context;
//use crate::models::event::EventWithSignups as Event;
use crate::models::event::Event;
use crate::models::signup::Signup;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use juniper::{graphql_object, FieldResult};

graphql_object!(Event: Context |&self| {
    description: "Metadata about an event"

    field id() -> i32 { self.id }

    field title() -> &str { self.title.as_str() }

    field description() -> &str { self.description.as_str() }

    field short_description() -> &str { self.short_description.as_str() }

    field background() -> &str
        as "URL to an image" { self.background.as_str() }

    field location() -> &str { self.location.as_str() }

    field start_time() -> DateTime<Utc> { self.start_time }

    field end_time() -> DateTime<Utc> { self.end_time }

    field price() -> i32 { self.price }

    field published() -> bool
        as "Whether the event is viewable to the public" { self.published }

    field signup_count(&executor) -> FieldResult<i32>
        as "Number of signed up attendees" {
        use crate::schema::tables::event_signups::dsl::*;
        let connection = executor.context().pool.get()?;
        let signups: Vec<Signup> = event_signups
            .filter(event.eq(self.id))
            .load(&connection)?;
        Ok(signups.len() as i32)
    }

    field signups(&executor) -> FieldResult<Vec<Signup>>
        as "Get information on all signed up attendees. \
            Requires authorization." {
        use crate::schema::tables::event_signups::dsl::*;
        gql_auth!(executor, Events(SignupRead))?;
        let connection = executor.context().pool.get()?;
        Ok(event_signups
            .filter(event.eq(self.id))
            .load(&connection)?)
    }
});

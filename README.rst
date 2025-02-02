strecklistan
============

This is the backend of strecklistan: a simple web-shop.

It is entirely built in Rust_. The backend uses the Rocket_
web server framework. Database integration and migrations
are provided by Diesel_.

The frontend is built in the Seed_ framework.
Trunk_ is used for bundling the frontend.

.. _Rust:   https://www.rust-lang.org/
.. _Rocket: https://rocket.rs/
.. _Diesel: https://diesel.rs/
.. _Seed:   https://seed-rs.org/
.. _Trunk:  https://trunkrs.dev/


Local Development
-----------------

Install the latest version of Rust using rustup_.

.. _rustup: https://rustup.rs/

The frontend and the backend need to be individually compiled. See
the guides for each. Or alternatively, you can use docker-compose
for a quick setup.

Quick setup using Docker
^^^^^^^^^^^^^^^^^^^^^^^^

You know what to do: ::

    # launch the app on :8080 and launch adminer on :8081.
    docker-compose up

    # and clean up when you're done
    docker-compose down


Frontend
^^^^^^^^

You'll need ``trunk``, and the webassembly compiler target.

Quick setup guide: ::

    # Install the WebAssembly target
    rustup target add wasm32-unknown-unknown

    # Install trunk
    cargo install -f --locked trunk

    cd frontend

    # Lanch a server to build and host the frontend
    trunk serve # listens on :8080

The trunk server will proxy api requests to the backend
**Make sure the backend is running if you want to try the app.**


Backend
^^^^^^^

All commands in this section are to be run in the ``backend``-folder.

You'll need the Postgres C/C++ client library ``libpq``, as well as
``openssl``. Install these via your preferred package manager.

You will also need a PostgreSQL database to connect to. Make sure
that you have one running. For a quick setup you can use docker. ::

    docker run --name "postgres" -d \
        --publish 5432:5432 \
        --env POSTGRES_PASSWORD=password \
        --env POSTGRES_USER=postgres \
        postgres:13

Then make sure to setup your local ``.env``-file with the proper
secrets and database settings. ::

    cp example.env .env

    $EDITOR .env

For handling database migrations you need to use the ``diesel`` CLI. ::

    cargo install diesel_cli --no-default-features --features "postgres"

    diesel setup # Create database

    diesel migration run # Run migrations, generate rust bindings

There is some mock data that you can use to populate the database
in the ``backend/db_mock/``-folder. If you use the example setup
the script ``populate.sh`` will do the work for you.

You can then run the application using cargo. ::

    # Compile and run
    cargo run

    # Run tests
    cargo test

    # Automatically recompile and run on file changes
    cargo install -f cargo-watch
    cargo watch -x run


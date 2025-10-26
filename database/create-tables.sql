DROP TABLE IF EXISTS stocks CASCADE;


DROP TABLE IF EXISTS stocks_history CASCADE;


DROP TABLE IF EXISTS users CASCADE;


DROP TABLE IF EXISTS accounts CASCADE;


DROP TABLE IF EXISTS user_stocks CASCADE;


-- TODO: Review the NOTE's;
-- TODO: What are sequence tables;
-- TODO: Consider using separate table for the price of the stock;
--  => CREATE TABLE stock_prices (
--     id SERIAL PRIMARY KEY,
--     stock_id INTEGER NOT NULL REFERENCES stocks(id),
--     price REAL NOT NULL,
--     recorded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
-- ); Something like that;
-- 
--
--
CREATE TABLE stocks (
    -- In an INSERT command, if ALWAYS is selected, a user-specified value is only accepted if the INSERT 
    -- statement specifies OVERRIDING SYSTEM VALUE.
    id SERIAL PRIMARY KEY,
    -- I think that is unique
    -- Maybe there are 2 companies with the same name, I think that might happen.
    -- NOTE: Maybe we should limit that to some length and use the VARCHAR
    -- But we either way need to lay some limit while inserting data, and that would be 
    -- backend logic that would handle that.
    -- I believe that is also called ticker.
    abbreviation TEXT NOT NULL UNIQUE,
    -- If there would be a company table that would be a foreign key to that table.ADD
    -- and things like company name would be located there.
    company TEXT NOT NULL,
    -- That is the date when the stock started trading. Not when it was creates inside in our system.
    since DATE NOT NULL,
    -- There are not unsigned types in Postgres
    price REAL NOT NULL CHECK (price > 0),
    -- That would represent percent change in price since the last data revalidation (ideally 1 minute).
    -- NOTE: That is not ideal is it is not explicitly stated that it is percent change.
    delta REAL NOT NULL CHECK (
        delta > -100
        AND delta < 100
    ),
    -- NOTE: Think about using TIMESTAMPTZ if we want to handle timezones.
    -- Use as TIMESTAMP WITH TIME ZONE if opt in to time zone support.
    last_update TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP -- 
    -- More fields like tickers, exchange, country, sector, industry, etc. could be added here
    -- Also since we are linking the stocks to the companies maybe we want a table about companies.
    -- Not sure how that would be useful though.
);


CREATE TABLE stocks_history (
    id SERIAL PRIMARY KEY,
    -- ON DELETE CASCADE means that if the stock is deleted from the stocks table, all its history
    -- will also be deleted. It does not work the other way around which is what we want.
    -- NOTE: Maybe we do not want the ON DELETE CASCADE, technically that would 
    -- waste our history, and that is not ideal.
    stock_id INTEGER NOT NULL UNIQUE REFERENCES stocks(id) ON DELETE CASCADE,
    -- Technically that cannot be NULL and it cannot be empty, as saying the 
    -- stock_id is NOT NULL means that there is at least one price.
    -- NOTE: Consider using that: CHECK (array_length(prices, 1) > 0),
    prices REAL [] NOT NULL
);


CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    -- NOTE: We will currently opt out of the user_tag. I think it should be moved in the users table.
    -- user_tag TEXT NOT NULL UNIQUE,
    -- More settings could be added here
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP --
    -- Create fields like password_hash, email, is_active, is_admin, last_login, etc. here.
);


CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    -- stocks INTEGER NOT NULL REFERENCES stocks(id) [] NOT NULL,
    -- stocks INTEGER [] NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    account_id INTEGER NOT NULL UNIQUE REFERENCES accounts(id),
    -- NOTE: balance can be negative, that is correct. Most of time balance will be negative as
    -- this is what happens when you play with other people money.
    balance REAL NOT NULL,
    -- That would represent the percent change in the user's portfolio value
    -- since the last data revalidation as the stocks may change their prices.
    delta REAL NOT NULL CHECK (
        delta > -100
        AND delta < 100
    )
);


-- That is junction table between users and stocks.
-- Is uses a composite primary key (user_id, stock_id) to retrieve the user related stocks.
-- To get all stocks of some user you would do:
-- SELECT stock_id FROM user_stocks WHERE user_id = some_user_id;
-- Or, per wikipedia: 
-- SELECT * FROM Users
-- JOIN UserPermissions USING (UserLogin);
CREATE TABLE user_stocks (
    -- That should not be SERIAL as it's value dependents on users(id) and stocks(id).
    -- We are just referencing those.stock_id
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stock_id INTEGER NOT NULL REFERENCES stocks(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, stock_id)
);


INSERT INTO
    accounts (created_at)
VALUES
    (CURRENT_TIMESTAMP) RETURNING id AS account_id;


-- Test user_stocks insertion
INSERT INTO
    users (account_id, balance, delta)
VALUES
    (1, 1000.0, 0.0) RETURNING id AS user_id;


-- Assume the returned id is 1
-- INSERT INTO
--     user_stocks (user_id, stock_id)
-- VALUES
--     (1, 1);
SELECT
    *
FROM
    users;


SELECT
    *
FROM
    accounts;


INSERT INTO
    stocks (abbreviation, company, since, price, delta)
VALUES
    ('AAPL', 'Apple Inc.', '1980-12-12', 150.0, 0.5) RETURNING id AS stock_id;


DO $$  
    BEGIN FOR i IN 1..40 LOOP
    INSERT INTO
        stocks (abbreviation, company, since, price, delta)
    VALUES
        (
            'GOOGL' || i,
            'Alphabet Inc.',
            '2004-08-19',
            2800.0,
            -0.3
        );
    END LOOP;
END $$;



SELECT
    *
FROM
    stocks;


INSERT INTO
    user_stocks (user_id, stock_id)
VALUES
    (1, 1);


-- INSERT INTO
--     user_stocks(user_id, stock_id)
-- VALUES
--     (1, 1) ON CONFLICT DO NOTHING;
INSERT INTO
    user_stocks (user_id, stock_id)
VALUES
    (1, 1),
    (1, 2) ON CONFLICT DO NOTHING;


SELECT
    *
FROM
    user_stocks;


SELECT
    *
FROM
    user_stocks
    JOIN stocks ON stocks.id = user_stocks.stock_id
    JOIN users ON users.id = user_stocks.user_id;
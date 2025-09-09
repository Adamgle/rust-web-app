# Stock Trading App – Component Notes

---

## Homepage Design

## Component: `Logo`

- Currently undefined, just a placeholder (string).
- Future: replace with an actual logo component.

---

## Component: `Profile`

- **Content**
  - User’s image.
  - Dropdown menu for profile actions.
  - Amount of money user has.
  - Indication of gain/loss since last login.
- **Navigation**
  - Link to the user’s profile page.
- **Profile Page**
  - Could be public if user opts in.
  - Might show portfolio (undecided).

---

## Component: `Sidebar`

- **Purpose**

  - Navigation to other app functionalities.
  - Only makes sense when the user is logged in.
  - If not logged in → keep sidebar visible but redirect to login on click.

- **Main Features**

  1. Your Stocks → detailed list of owned stocks.
  2. Trade → trade with other users on the platform.
  3. Trading Actions → automated trading rules (TBD).
  4. Settings → account configuration and preferences.
  5. Watchlist (optional) → track stocks user doesn’t own.
     - Private page, only accessible when logged in.

- **Advanced / Nice-to-Have**
  1. Market → browse trending stocks, news, sectors, performance.
  2. Analytics / Insights → charts, performance breakdowns, risk analysis.
     - Would require stock analysis algorithms.

---

## Component: `Stocks`

- **Display**
  - Stocks in a grid format.
  - Each card shows:
    - Stock name.
    - Current price.
    - Up/down trend since some time.
- **Interaction**
  - Click on stock → open detail view.
  - Detail view may include:
    - Stock chart.
    - Additional information.

# Feature implementation details/overview

### Your Stocks

### Trade

Allow you to trade with other users.

Requires:

- Database with users defined

### Trading Actions

Automatic trading rules.

### Market

That would require us to set up the database. We need to store that stocks, but also make sure the stocks are updated frequently, we have define the limit to be 1 minute delay of the stocks data refresh.
Market will allow users to buy stocks, each stock should also contain the details page of a single stock, like history prices, chart of the history prices, rest is TBD, but plenty to do here.

### Watchlist

Add stock to watchlist, something like notification system would be nice about the watched stocks.

### Settings

User account related settings.

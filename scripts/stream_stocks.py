# # import yfinance as yf
# # from pprint import pprint

# # ticker = yf.Ticker("AAPL")

# # import yfinance as yf

# # dat = yf.Ticker("MSFT")

# # dat = yf.Ticker("MSFT")

# # pprint(dat.info)
# # pprint(dat.calendar)
# # pprint(dat.analyst_price_targets)
# # pprint(dat.quarterly_income_stmt)
# # pprint(dat.history(period="1mo"))
# # pprint(dat.option_chain(dat.options[0]).calls)


# import datetime
# import json
# import pprint
# from typing import Any, Dict
# import matplotlib.pyplot as plt
# from pandas import DataFrame
# import seaborn as sns
# import yfinance as yf


# class PolandWinterTime(datetime.tzinfo):
#     def utcoffset(self, dt):
#         return datetime.timedelta(hours=1)

#     def dst(self, dt):
#         return datetime.timedelta(0)

#     def tzname(self, dt):
#         return "+01:00"

#     def __repr__(self):
#         return f"{self.__class__.__name__}()"


# # set the start and end dates for our market data request
# now = datetime.datetime.now(tz=PolandWinterTime())

# end_date = now
# # start_date = now - datetime.timedelta(days=365 * 2)
# start_date = now - datetime.timedelta(days=4)

# formatting = "%Y %H:%M:%S %Z"

# print("Using: ", (start_date.strftime(formatting), end_date.strftime(formatting)))

# ticker = "NVDA META AAPL"


# def get_stock(ticker: str, interval: str = "2d"):
#     with open("stocks.json", "w+", encoding="UTF-8") as f:
#         # download market data for a single ticker
#         df_single = yf.download(
#             tickers=ticker,
#             start=start_date,
#             end=end_date,
#             interval=interval,
#             group_by="ticker",
#             progress=False,
#         )

#         data = None

#         if df_single is not None:
#             # d = f.read()
#             # # Get the previous content or start with empty
#             # data = DataFrame(data=json.loads(d)) if d else DataFrame(data={})
#             # f.seek(0)

#             # data.update(df_single)
#             # print(data)

#             # f.write(data.to_json())

#             f.write(df_single.to_json())

#     return df_single


# print(get_stock(ticker, interval="1m"))


# import yfinance as yf


# # define your message callback
# def message_handler(message):
#     print("Received message:", message)


# # =======================
# # With Context Manager
# # =======================
# with yf.WebSocket() as ws:
#     ws.subscribe(["AAPL", "BTC-USD"])
#     ws.listen(message_handler)

import asyncio
from contextlib import redirect_stdout
import datetime
import json
import os
import sys
import yfinance as yf
import logging
from consts import TICKERS

# Usually seen fields in stocks schema
stock_keys = {
    # "market_hours",
    # "exchange",
    "time",
    # "quote_type",
    "price",
    # "price_hint",
    "id",
    # Those 2 where not usually seen, they should be treated as option from the stream ticker, but can be derived from the previous values
    "change_percent",
    "change",
}


# define your message callback
def message_handler(message):
    global stock_keys

    # Tickers comes as a dict and we only care about that, we are not processing anything else.
    if isinstance(message, dict):
        # keys = set(message.keys())

        # if union is None:
        #     union = keys
        # else:
        #     union &= keys

        ticker = {k: message[k] for k in stock_keys if k in message}

        if "time" in ticker:
            dt = datetime.datetime.fromtimestamp(int(ticker["time"]) / 1000)
            ticker["time"] = dt.isoformat()  # Standard ISO format: 2026-04-24T21:32:34

        # Test, to seen what tickers do not have all of those usually seen fields and which are lacking

        diff = stock_keys - message.keys()

        if diff:
            with open("./logs/missing_fields.txt", mode="a", encoding="UTF-8") as f:
                f.write(
                    f"[{datetime.datetime.now()}] Ticker {message["id"]} are missing fields: {str(sorted(diff))} | ticker = {message.keys()}\r\n"
                )

        try:
            ticker = json.dumps(ticker)
        except Exception as e:
            # TODO: Maybe that should write to some log file that some stock tickers are not sent to the server
            # as they are no serializable, although that should not happen.
            print(f"Could not serialize to JSON: {e}", file=sys.stderr)

        print(ticker, flush=True)


async def main():
    # Usually, crypto currencies have different schema than stocks
    # "BTC-USD",

    # =======================
    # With Context Manager
    # =======================
    async with yf.AsyncWebSocket(verbose=False) as ws:
        await ws.subscribe(TICKERS)
        await ws.listen(message_handler=message_handler)

    # =======================
    # Without Context Manager
    # =======================

    # ws = yf.AsyncWebSocket()
    # await ws.subscribe(["AAPL", "BTC-USD"])
    # await ws.listen()


# from contextlib import redirect_stdout
# import io

# f = io.StringIO()
# with redirect_stdout(f):
#     help(pow)
# s = f.getvalue()


asyncio.run(main())

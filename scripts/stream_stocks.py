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
import json
import yfinance as yf

union = None


# {'market_hours', 'price', 'exchange', 'change', 'time', 'quote_type', 'price_hint', 'id', 'change_percent'}
# {'market_hours', 'price', 'exchange', 'change', 'time', 'quote_type', 'price_hint', 'id', 'change_percent'}
# Mutual fields: {'market_hours', 'price', 'exchange', 'price_hint', 'quote_type', 'time', 'id'}
# {'market_hours', 'exchange', 'time', 'quote_type', 'price', 'price_hint', 'id'}


# define your message callback
def message_handler(message):
    global union

    # Tickers comes as a dict and we only care about that, we are not processing anything else.
    if isinstance(message, dict):
        # keys = set(message.keys())

        # if union is None:
        #     union = keys
        # else:
        #     union &= keys

        ticker = json.dumps(message)
        print(ticker, flush=True)


async def main():
    tickers = [
        "AAPL",
        # "MSFT",
        # "NVDA",
        # "TSLA",
        # "AMZN",
        # "GOOGL",
        # "META",
        # "SPY",
        # "QQQ",
        # "BTC-USD",
    ]

    # =======================
    # With Context Manager
    # =======================
    async with yf.AsyncWebSocket() as ws:
        await ws.subscribe(tickers)
        await ws.listen(message_handler=message_handler)

    # =======================
    # Without Context Manager
    # =======================

    # ws = yf.AsyncWebSocket()
    # await ws.subscribe(["AAPL", "BTC-USD"])
    # await ws.listen()


asyncio.run(main())

import yfinance as yf
from pprint import pprint

ticker = yf.Ticker("AAPL")

pprint(ticker.history(period="1y"))

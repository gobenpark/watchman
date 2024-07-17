from pykrx import stock
from datetime import datetime
from dateutil.relativedelta import relativedelta
from talib import abstract
import FinanceDataReader as fdr
from functools import lru_cache


class Envolope:

    def __init__(self):
        self.dfs = {}

    def get_top100_amount_ticker(self, date: str) -> list:
        """
        거래대금 상위 100개 종목을 가져온다.
        :param date: 날짜
        :return: 100개의 종목 리스트
        """
        df = stock.get_market_cap(date)

        df.sort_values(by='거래대금', ascending=False, inplace=True)
        return list(df[:100].index.tolist())

    def update_df(self,ticker):
        end = datetime.now().strftime('%Y-%m-%d')
        start = datetime.now() - relativedelta(years=1)
        df = fdr.DataReader(ticker, start, end)
        df['prev_close'] = df['Close'].shift(1)
        df['diff'] = (df['Close'] - df['prev_close']) / df['prev_close'] * 100
        df['moving20'] = abstract.SMA(df, timeperiod=20, price='Close')
        df['moving10'] = abstract.SMA(df, timeperiod=10, price='Close')
        df['moving7'] = abstract.SMA(df, timeperiod=7, price='Close')
        df['upper'] = df['moving20'] * 1.4
        df['mask_upper_cross'] = df['High'] > df['upper']
        self.dfs[ticker] = df

    @lru_cache(maxsize=128)
    def target(self):
        end = datetime.now().strftime('%Y-%m-%d')
        start = datetime.now() - relativedelta(years=1)
        tk = self.get_top100_amount_ticker(end)
        items = []
        for i in tk:
            df = fdr.DataReader(i, start, end)
            df['prev_close'] = df['Close'].shift(1)
            df['diff'] = (df['Close'] - df['prev_close']) / df['prev_close'] * 100
            df['moving20'] = abstract.SMA(df, timeperiod=20, price='Close')
            df['moving10'] = abstract.SMA(df, timeperiod=10, price='Close')
            df['moving7'] = abstract.SMA(df, timeperiod=7, price='Close')
            df['upper'] = df['moving20'] * 1.4
            df['mask_upper_cross'] = df['High'] > df['upper']
            rows_upper_cross = df[df['mask_upper_cross']]
            close = df['Close'].iloc[-1]
            mv7 = df['moving7'].iloc[-1]
            v = abs(((close - mv7) / mv7) * 100)
            if rows_upper_cross.any(axis=None):
                if 2 > v:
                    items.append([i, stock.get_market_ticker_name(i)])
            self.dfs[i] = df
        print(items)
        return [i[0] for i in items]

    def buy(self, ticker, current_price) -> bool:
        if ticker not in self.dfs:
            self.update_df(ticker)
        df = self.dfs[ticker]
        mv7 = df['moving7'].iloc[-2]
        v = abs(((current_price - mv7) / mv7) * 100)
        if v < 0.5:
            return True
        return False

    def sell(self, ticker, order_price, current_price) -> bool:
        if ticker not in self.dfs:
            self.update_df(ticker)
        df = self.dfs[ticker]

        if order_price > current_price:
            mv20 = df['moving20'].iloc[-1]
            v = abs(((current_price - mv20) / mv20) * 100)
            if v < 0.5:
                return True
        else:
            mv10 = df['moving10'].iloc[-1]
            v = abs(((current_price - mv10) / mv10) * 100)
            if v < 0.5:
                return True
        return False

    def name(self) -> str:
        return "envolope"

if __name__ == '__main__':
    e = Envolope()
    # print(e.buy("028300",108150))
    print(e.target())

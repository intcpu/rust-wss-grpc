import time

import signal_pb2 as pb
import signal_pb2_grpc as pbg
from client_wrapper import ServiceClient


def run():
    users = ServiceClient(pbg, 'SignalStub', 'localhost', 50051)
    # Insert example metadata
    metadata = [('ip', '127.0.0.1')]
    res = users.GetBookTickers(pb.BookTickerReq(exchange='BINANCE', pairs=["XRPUSDT", "BTCUSDT"]), metadata)
    print('res:', time.time(), res)


if __name__ == '__main__':
    run()

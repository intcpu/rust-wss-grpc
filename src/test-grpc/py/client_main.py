import signal_pb2 as pb
import signal_pb2_grpc as pbg
from client_wrapper import ServiceClient


def run():
    users = ServiceClient(pbg, 'SignalStub', 'localhost', 50051)
    # Insert example metadata
    metadata = [('ip', '127.0.0.1')]
    res = users.GetBookTickers(pb.BookTickerReq(exchange='BINANCE', pairs=["XRP_USDT"]), metadata)
    print('res:', res)


if __name__ == '__main__':
    run()

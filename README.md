DoH
==========
Query the host IP address by DoH(DNS over HTTPs)


## Install 
```bash
$ cargo install --path .
```

## Usage

```bash
$ doh --help
Query the host IP address by DoH(DNS over HTTPs)

Usage: doh.exe --host <HOST>

Options:
      --host <HOST>  Query hostname
  -h, --help         Print help
```


```bash
$ doh --host github.com
┌─────────┬─────────────┬──────┬─────┬────────────────┬───────┬──────┐
│   DoH   │    Name     │ Type │ TTL │    Address     │  Avg  │ Lost │
├─────────┼─────────────┼──────┼─────┼────────────────┼───────┼──────┤
│ 1.1.1.1 │ github.com  │ 1    │ 40  │ 192.30.255.113 │ 180ms │   0% │
├─────────┼─────────────┼──────┼─────┼────────────────┼───────┼──────┤
│ 9.9.9.9 │ github.com. │ 1    │ 22  │ 140.82.121.4   │ 254ms │  10% │
├─────────┼─────────────┼──────┼─────┼────────────────┼───────┼──────┤
│ aliyun  │ github.com. │ 1    │ 1   │ 20.205.243.166 │     / │ 100% │
└─────────┴─────────────┴──────┴─────┴────────────────┴───────┴──────┘
```
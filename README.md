# cron_gate

```sh
$ ./cron_gate -h
cron_gate 0.1.0
miyanokomiya


USAGE:
    cron_gate [OPTIONS] <expression>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --after <after>      Dates after 'Y/m/d H:M:S'
    -n, --number <number>    Displayed number [default: 10]

ARGS:
    <expression>    Cron Expression '* * * 7 * command'

```

```sh
$ ./cron_gate "0 0 * * 1 echo 'Yo'" -n 10 -a "2019/01/01 0:0"
2019/01/07 00:00 echo 'Yo'
2019/01/14 00:00 echo 'Yo'
2019/01/21 00:00 echo 'Yo'
2019/01/28 00:00 echo 'Yo'
2019/02/04 00:00 echo 'Yo'
2019/02/11 00:00 echo 'Yo'
2019/02/18 00:00 echo 'Yo'
2019/02/25 00:00 echo 'Yo'
2019/03/04 00:00 echo 'Yo'
2019/03/11 00:00 echo 'Yo'
```

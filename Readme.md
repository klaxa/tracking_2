Track yo'self 2
===============

This set of programs tracks your i3 window focus and stores it in an sqlite3 file-backed database.
This project is the successor project of [klaxa/tracking](https://github.com/klaxa/tracking).

Usage
-----

`tracking` is meant to be running at all times. It queries i3 through `i3-msg`
every 10 seconds and stores the focused window's information in the database.
You can simply add it with `exec /path/to/tracking` in your i3 configuration
file.

The location of the database defaults to `tracking.db` in the current
directory. This value can either be set with the `-d` flag or by setting the
environment variable `TRACKING_DB` to the path you wish to use.

To compile the programs, simply clone the repository and run `cargo build` or `cargo build --release`

`gen_graph` creates daily graphs of the tracked data including some statistics.

The following usage message can also be obtained by running `gen_graph -h`:

```

Usage: gen_chart [OPTIONS]

Options:
  -d, --database <DATABASE>  The database to connect to, defaults to 'tracking.db' can also be set with TRACKING_DB environment variable
  -s, --start <START>        The start date in the format YYYY-MM-DD, defaults to today
  -e, --end <END>            The end date in the format YYYY-MM-DD, defaults to today
  -t, --today                Only generate graph for the start date
  -w, --week                 Only generate graph for the week containing start date
  -m, --month                Only generate graph for the month containing start date
  -i, --idle                 Inlcude idle time in graph
      --height <HEIGHT>      Height of the 24 hour portion of the graph, defaults to 500 px [default: 500]
  -h, --help                 Print help
  -V, --version              Print version
```

Example chart:

![chart](https://github.com/klaxa/tracking_2/assets/1451995/23fa427b-3f9a-4b36-b793-96203ab2f84d)

`screentime` uses the same combination of either using the `-d` flag, the envirnment variable or the fallback of `tracking.db` in the current directory and outputs the screentime for the current day with the top 3 types of programs and their corresponding screentime. Using the `-s` flag, a different start time from `0:00` can be chosen. The output of this program is suited for use in i3blocks. The following configuration can be used:

```
[screentime]
command=/path/to/screentime -d /path/to/tracking.db -s 6:00
interval=10
```


Example i3blocks output:

![i3blocks](https://github.com/klaxa/tracking_2/assets/1451995/8ce47530-84e4-4675-b1fc-a8466d4797e1)

# cjconv
## A CLI tool to convert between CSV and JSON formats

```

Usage: cjconv <COMMAND>

Commands:
  csv-to-json  Convert CSV to JSON
  json-to-csv  Convert JSON to CSV
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help

csv-to-json:

Convert CSV to JSON

Usage: cjconv csv-to-json [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>          Input CSV file
  -o, --output <OUTPUT>        Output JSON file
  -a, --array-format           Output as array of objects (default) or as array of arrays
  -d, --delimiter <DELIMITER>  CSV delimiter character (default: ,) [default: ,]
      --has-headers            CSV has headers (default: true)
      --trim                   Trim whitespace from fields (default: false)
  -h, --help                   Print help


json-to-csv:

Convert JSON to CSV

Usage: cjconv json-to-csv [OPTIONS] --input <INPUT> --output <OUTPUT>

Options:
  -i, --input <INPUT>          Input JSON file
  -o, --output <OUTPUT>        Output CSV file
  -d, --delimiter <DELIMITER>  CSV delimiter character (default: ,) [default: ,]
      --quote-all              Quote all non-numeric fields (default: false)
  -h, --help                   Print help

```


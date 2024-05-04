import json
import requests
import argparse
from pathlib import Path

parse = argparse.ArgumentParser(
    prog = "Coveralls summariser",
    description = "Interprets Cargo Tarpaulin coverage and main's coveralls status"
)

parse.add_argument("-m", required = True, action="store", dest = "main", help = "comparison branch name")
parse.add_argument("-r", required = True, action="store", dest = "repo", help = "repository uri, e.g. \"github/JerboaBurrow/Busser\"")
parse.add_argument("--human", required = False, action = "store_true", dest = "human", help = "adds additional humanised message about the result")

args = parse.parse_args()

status = requests.get(
    url = f"https://coveralls.io/{args.repo}.json?branch={args.main}",
    headers = {'content-type': 'application/json', 'Accept-Charset': 'UTF-8'}
)

main_coverage = None

if status.ok:
    try:
        data = status.json()
        if data is not None and 'covered_percent' in data:
            main_coverage = round(data['covered_percent'], 2)
    finally:
        pass

cov = Path('tarpaulin-report.json')
if not cov.is_file():
    if args.human:
        print(f"Hmm... Tarpaulin did not seem to generate a json coverage file at {cov}")
    else:
        print(f"No coverage file generated")
    exit(0)

entries = []
max_size = 0
covered = 0
coverable = 0
for file in json.load(open(cov))['files']:
    path = file['path']
    if file['coverable'] == 0 or 'tests' in path:
        continue
    
    coverage = round(100*float(file['covered'])/float(max(1, file['coverable'])),2)
    lines = f"{file['covered']} / {file['coverable']}"
    covered += file['covered']
    coverable += file['coverable']

    if 'src' in path:
        name = '/'.join(path[path.index('src'):len(path)])
    else:
        name = '/'.join(path)
 
    entries.append((name, coverage, lines))
    max_size = max(max_size, len(name))
    
entries = sorted(entries, key = lambda x: x[1])

print(covered, coverable)
this_coverage = round(100.0*covered/coverable, 2)

diff = None
if main_coverage is not None:
    diff = round(this_coverage-main_coverage, 2)


if args.human:
    header = ""
    if diff is not None:
        if diff > 0:
            header = f"Looks like you increased coverage by {diff} %, fantastic!\n"
        elif diff < 0:
            header = f"Seems the coverage is below {args.main} by {diff} %, please consider adding to the tests, thanks!\n"
        elif diff == 0:
            header = f"Great, coverage is exactly the same as {args.main}!\n"

    header += "Here is the full report breakdown\n\n"
else:
    header = ""

output = f"```\n{header}Total coverage {this_coverage} %"

if diff is not None:
    if diff < 0:
        sign = "-"
    elif diff > 0:
        sign = "+"
    else:
        sign = "" 

    output += f" ({sign}{abs(diff)} % against main)"

output += "\n"+"_"*max_size
for entry in entries:
    pad = " "*(max_size-len(entry[0]))
    cov = str(entry[1])
    if len(cov) < 5:
        cov += " "*(5-len(cov))

    output += "\n"+entry[0]+pad+" | "+cov + " %"
    output += " " + entry[2]
output += "\n"+"_"*max_size+"\n```"
print(output)
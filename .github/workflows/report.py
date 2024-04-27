import json
import requests
import argparse

parse = argparse.ArgumentParser(
    prog = "Coveralls summariser",
    description = "Interprets Cargo Tarpaulin coverage and main's coveralls status"
)

parse.add_argument("-m", required = True, action="store", dest = "main", help = "comparison branch name")
parse.add_argument("-r", required = True, action="store", dest = "repo", help = "repository uri, e.g. \"github/JerboaBurrow/Busser\"")
parse.add_argument("-h", required = False, action = "store_true", dest = "human", help = "adds additional humanised message about the result")

args = parse.parse_args()

status = requests.get(
    url = f"https://coveralls.io/{args.repo}.json?branch={args.main}",
    headers = {'content-type': 'application/json', 'Accept-Charset': 'UTF-8'}
)

main_coverage = None

if status.ok:
    try:
        data = status.json()
        if 'covered_percent' in data:
            main_coverage = round(data['covered_percent'], 2)
    finally:
        pass

entries = []
max_size = 0
overall = 0.0
for file in json.load(open('tarpaulin-report.json'))['files']:
    coverage = round(100*float(file['covered'])/float(max(1, file['coverable'])),2)
    overall += coverage
    path = file['path']
    if 'src' in path:
        name = '/'.join(path[path.index('src'):len(path)])
    else:
        name = '/'.join(path)
 
    entries.append((name, coverage))
    max_size = max(max_size, len(name))
    
entries = sorted(entries, key = lambda x: x[1])

this_coverage = round(overall/len(entries),2)

diff = None
if main_coverage is not None:
    diff = this_coverage-main_coverage


if args.human:
    header = ""
    if diff is not None:
        if diff > 0:
            header = f"Looks like you increased coverage by {diff} %, fantastic!"
        elif diff < 0:
            header = f"Seems the coverage is below {args.main} by {diff} %, please consider adding to the tests, thanks!"
        elif diff == 0:
            header = f"Great, coverage is exactly the same as {args.main}!"

    header += "\nHere is the full report breakdown\n\n"
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

    output += " ({sign}{diff} against main)"

output += "\n"+"_"*max_size
for entry in entries:
    pad = " "*(max_size-len(entry[0]))
    output += "\n"+entry[0]+pad+" | "+str(entry[1]) + " %"
output += "\n"+"_"*max_size+"\n```"
print(output)
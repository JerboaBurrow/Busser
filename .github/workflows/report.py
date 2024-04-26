import json

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

print(f"```\nTotal coverage {round(overall/len(entries),2)} %")
print("_"*max_size)
for entry in entries:
    pad = " "*(max_size-len(entry[0]))
    print(entry[0]+pad+" | "+str(entry[1]) + " %")
print("_"*max_size+"\n```")
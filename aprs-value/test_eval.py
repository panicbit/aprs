import json
import math
import sys

code = sys.argv[1]

def bail(msg):
    if isinstance(msg, Exception):
        msg = msg.args[0]
    value = { 'err': msg }
    value = json.dumps(value)
    print(value)
    sys.exit()

def try_(f):
    try:
        return f()
    except Exception as e:
        bail(e)

def tag(value):
    match value:
        case int(): return { 'int': value }
        case float():            
            if math.isnan(value):
                value = 'NaN'
            elif value == float('inf'):
                value = 'inf'
            elif value == float('-inf'):
                value = '-inf'
            
            return { 'float': value }
        case bool(): return { 'bool': value }
        case str(): return { 'str': value }
        case list(): return { 'list': [ tag(item) for item in value ] }
        case dict(): return { 'dict': [ (tag(k), tag(v)) for k, v in value.items() ] }
        case set(): return { 'set': [ tag(item) for item in value ] }
        case tuple(): return { 'tuple': [ tag(item) for item in value ] }
        case _: raise Exception(f"tag: unhandled type: {type(value)}")

try:
    value = eval(code)
except Exception as e:
    bail(e.args[0])

tagged_value = tag(value)
result = { "ok": tagged_value }
result = json.dumps(result)

print(result)
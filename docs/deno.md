# Deno Support

Deno is detected on a deno.json file

Additionally, deno is detected if any `.js` or `.ts` file is found that imports something from `deno.land`.

**Install**:

_None_

**Build**

_None_

**Start**
If a deno.json is present
```
deno task start
```

Otherwise
```
deno run --allow-all index.j|ts
```

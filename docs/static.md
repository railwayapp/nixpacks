# StaticHTML Support

HTML will be served via Nginx if a `Staticfile` OR a `public` folder OR an `index` folder OR a `dist` folder OR an `index.html` file is found.

**Install**:

```
Skipped
```

**Build**

```
mkdir /etc/nginx/ /var/log/nginx/ /var/cache/nginx/
```

**Start**

```
nginx -c {conf} .
```

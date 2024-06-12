# alpha.42

**We're now pre-compiling CSS for faster startup times**

We are beginning to optimize the compilation and dev startup performance of Payload 3.0, and the first thing we've done is pre-compiled all SCSS that is required by the Payload admin UI.

If you started before `alpha.42` and update to this version, you'll see that no styles are loaded. To fix this, take a look at the `/src/app/(payload)/layout.tsx` file within this repo, and note that this file has a new CSS import within it to load all CSS required by the Payload admin panel.

Simply add this import to your `(payload)/layout.tsx` file, and you'll be back in business.

**We now load all required Payload static files from your Next.js `public` folder**

In addition to pre-compiling SCSS, we are also shipping static files through the Next.js `/public` folder which will cut down on compilation time as well. Make sure you copy the `public/payload` folder into your repo as well as add the CSS import above.

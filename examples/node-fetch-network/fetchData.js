import fetch from 'node-fetch';

const url = process.env.REMOTE_URL;

if (!url) {
    console.error('REMOTE_URL is not defined in the environment variables.');
    process.exit(1);
}

fetch(url)
    .then(response => {
        if (!response.ok) {
            throw new Error(`Network response was not ok: ${response.statusText}`);
        }
        return response.statusText;
    })
    .then(data => {
        console.log('Fetched data:', data);
    })
    .catch(error => {
        console.error('Fetching data failed:', error);
        process.exit(1);
    });
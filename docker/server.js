const { networkInterfaces } = require('os');

const express = require('express');
const app = express();
const port = 8085;


app.use(express.static('../docs/html'))


app.listen(port, '::', () => {

  loop1: for (key of Object.keys(networkInterfaces()).reverse()) {

    if (['lo', '127.0.0.1', '::1'].indexOf(key) >= 0)
      continue loop1;

    loop2: for (net of networkInterfaces()[key].reverse()) {

      if (net['family'] == 'IPv4') 
        addr = net['address'];

      else if (net['family'] == 'IPv6' && net['scopeid'] == 0) 
        addr = `[${net['address']}]`;

      else 
        continue loop2;

      console.log(`Docs available at http://${addr}:${port}`);

      break loop1;
    }
  }

})

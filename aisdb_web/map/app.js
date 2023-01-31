if ('caches' in window) {
  caches.keys()
    .then((keyList) => {
      return Promise.all(keyList.map((key) => {
        return caches.delete(key);
      }));
    });
}
navigator.serviceWorker.getRegistrations().then((registrations) => {
  for (let registration of registrations) {
    registration.unregister();
  }
});

// import { init_maplayers } from './map';

window.addEventListener('load', async () => {
  let [
    { createVesselMenuItem, vesselmenu, vesseltypeselect },
    { vessellabels },
  ] = await Promise.all([
    import('./selectform'),
    import('./palette'),
  ]);

  let { init_maplayers } = await import('./map');
  init_maplayers();

  createVesselMenuItem('All', 'All', '⋀');
  for (let label of vessellabels) {
    createVesselMenuItem(label, label);
  }
  createVesselMenuItem('Unknown', 'None', '○');
  vesseltypeselect.onclick = function() {
    vesselmenu.classList.toggle('show');
  };

  await import('./livestream.js');
});

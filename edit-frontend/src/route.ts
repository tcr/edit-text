export function pageId(): string {
  return window.location.pathname.match(/^\/?([^\/]+)/)![1] || '';
}

export function clientProxyUrl(): string {
  return '' +
    (window.location.protocol.match(/^https/) ? 'wss://' : 'ws://') +
    window.location.host.replace(/\:\d+/, ':8002') +
    '/' +
    pageId();
}

export function syncUrl(): string {
  return '' +
    (window.location.protocol.match(/^https/) ? 'wss://' : 'ws://') +
    (window.location.host.match(/localhost|0.0.0.0/) ?
      window.location.host.replace(/:\d+$|$/, ':8001') + '/$/ws/' + pageId() :
      window.location.host + '/$/ws/' + pageId());
}

export function graphqlUrl(): string {
  return '' +
    window.location.protocol + '//' +
    (window.location.host.match(/localhost|0.0.0.0/) ?
      window.location.host.replace(/:\d+$|$/, ':8003') + '/graphql/' :
      window.location.host + '/$/graphql/');
}

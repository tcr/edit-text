// Hashtag state

export default class HashState {
  static get(): Set<String> {
    return new Set((location.hash || '')
      .replace(/^#/, '')
      .split(',')
      .map(x => x.replace(/^\s+|\s+$/g, ''))
      .filter(x => x.length));
  }

  static set(input: Set<String>) {
    location.hash = Array.from(input).join(',');
  }
}

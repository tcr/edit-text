export function textNodeAtPoint(
  x: number,
  y: number,
): {textNode: Text, offset: number} {
  let textNode, offset;
  if ((<any>document).caretPositionFromPoint) {
    let range = (<any>document).caretPositionFromPoint(x, y);
    textNode = range.offsetNode;
    offset = range.offset;
  } else if (document.caretRangeFromPoint) {
    let range = (<any>document).caretRangeFromPoint(x, y);
    textNode = range.startContainer;
    offset = range.startOffset;
  } else {
    return null;
  }

  // TODO: can textNode ever be an element?
  if (textNode.nodeType !== 3) {
    return null;
  }

  return {
    textNode,
    offset,
  };
}

export function matchesSelector(
  el: Node,
  selector: String,
): boolean {
  return (<any>el).mozMatchesSelector(selector);
}

export function convert_data(data) {
    const props = [
      'Background',
      'Head',
      'Expression',
      'Mouth',
      'Top',
      'Face',
    ];
  
    let item = {
      id: data.tokenId,
      url: '/Token/'+data.tokenId,
      name: 'Wojak #' + data.tokenId,
      desc: '',
      properties: []
    }
  
    for (let i = 0; i < props.length; i++) {
      let name = props[i];
      let value = data[name];
  
      item.properties.push({ 'name': name, 'value': value });
    }
  
    return item;
  }
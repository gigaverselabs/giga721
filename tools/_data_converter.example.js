export function convert_data(data) {
  const props = [
    'Background',
    'Body',
    'Nose',
    'Mouth',
    'Eyes',
    'Head',
    'Top',
  ];

  let item = {
    id: data.tokenId,
    url: '/Token/' + data.tokenId,
    name: 'ICPunks #' + data.tokenId,
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

import fs from 'fs';

export function getMetadata() {
  let metadata = JSON.parse(fs.readFileSync('../../json/_metadata.json'));

  let data = [];

  for (var i = 0; i < metadata.length; i++) {
    let item = convert_data(metadata[i]);

    data.push(item);
  }

  return data;
}
import { getActor } from "./_common.js";
import { convert_data } from "./_data_converter.js";
import fs from 'fs';


function get_image_path(id) {
    var path = '../data/test output/'+id;
  
    if (fs.existsSync(path + '.jpg')) return [path + '.jpg', 'image/jpg'];
    if (fs.existsSync(path + '.png')) return [path + '.png', 'image/png'];
  
    return [];
  }

async function run() {
    let actor = getActor(false);
    let metadata = JSON.parse(fs.readFileSync('../data/wojak-metadata.json'));

    let data = [];

    for (var i = 0; i < metadata.length; i++) {
        let item = convert_data(metadata[i]);

        data.push(item);
    }

    let wait = [];

    for (var i = 0; i < data.length; i++) {

        let item = data[i];

        try {
            var [imagePath, contentType] = get_image_path(item.id);

            let buffer = fs.readFileSync(imagePath);

            let data = {
                name: "/Token/"+item.id,
                content_type: contentType,
                data: [...buffer]
            }

            wait.push(actor.upload_asset(data));

            // let result = await actor.upload_asset(data);
            // console.log(result);
        }
        catch (e) {
            console.error(e);
        }

        if (wait.length >= 20) {
            let result = await Promise.all(wait);
            console.log(result);
        }
    }

    let result = await Promise.all(wait);
    console.log(result);
}

run();
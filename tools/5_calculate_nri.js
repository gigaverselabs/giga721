import { getMetadata } from "./_data_converter.js";
import BigNumber from 'bignumber.js';
import fs from 'fs';

async function run() {
    let data = getMetadata();

    let props = {};

    data.forEach((x) => {
        for (let i = 0; i < x.properties.length; i++) {
            let prop = x.properties[i];

            let name = prop['name'];
            let value = prop['value'];

            if (props[name] === undefined) props[name] = {};

            let cur = props[name][value];

            if (cur === undefined) {
                props[name][value] = { value: 1 };
            } else {
                props[name][value].value = cur.value + 1;
            }

        }

    })

    for (let prop in props) {
        for (let val in props[prop]) {

            let i = props[prop][val];

            i.probability = BigNumber(i.value).dividedBy(BigNumber(data.length));

        }
    }

    for (var i = 0; i < data.length; i++) {
        let item = data[i];

        let prob = BigNumber(1);

        for (var p = 0; p < item.properties.length; p++) {
            let prop = item.properties[p];

            let val = props[prop.name][prop.value].probability;

            prob = prob.multipliedBy(BigNumber(val));
        }

        item.probability = prob;

    }

    //Sorty by propability
    data.sort((x, y) => x.probability - y.probability);


    //Assign NRI
    // let step = BigNumber(1/data.length);
    // let nri = BigNumber(1);

    let rank = 1;
    let skipped = 0;

    data[0].rank = 1;
    data[0].nri = 1;

    for (var i = 1; i < data.length; i++) {
        let skip = false;

        let prev_prob = data[i-1].probability;
        let cur_prob = data[i].probability;

        skip = prev_prob.isEqualTo(cur_prob);

        skipped++;
        if (skip) {
            data[i].rank = rank;
        } else {
            rank += skipped;
            skipped = 0;
        }

        data[i].rank = rank;

        data[i].nri = getNri(data[i].rank, data.length);
    }

    //sort by token Id
    data.sort((x,y) => x.id - y.id);

    let nri = data.map((x) => x.nri);

    fs.writeFileSync('nri.json', JSON.stringify(nri));

}

function getNri(rank, length) {
    let a = BigNumber(rank-1).dividedBy(BigNumber(length));
    let nri = BigNumber(1).minus(a);
    let nrif = Number(nri.toFixed(4));

    return nrif;
}

// getNri(10000, 10000);

run();
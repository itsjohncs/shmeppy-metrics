function main() {
    Promise.all([
        fetch("data/convocations.json")
            .then(function(r) {
                return r.json()
            }),
        new Promise(function(resolve, reject) {
            try {
                google.charts.load("current", {packages: ["corechart", "bar"]});
                google.charts.setOnLoadCallback(resolve);
            } catch(e) {
                reject(e);
            }
        }),
    ]).then(function([convocations]) {
        drawUniqueGMsPerMonth(
            document.getElementById("chart_uniqueGMsPerMonth"),
            convocations);
    });
}


function drawUniqueGMsPerMonth(element, convocations) {
    function getNumUniqueGMs(keys) {
        const gmAccountIds = new Set();
        for (const k of keys) {
            for (const convocation of convocations[k]) {
                for (const gmAccountId of convocation.admins) {
                    gmAccountIds.add(gmAccountId);
                }
            }
        }

        return gmAccountIds.size;
    }

    const monthToKeys = new Map();
    for (const k of Object.keys(convocations)) {
        const month = /^([0-9]{4}-[0-9]{2})-[0-9]{2}$/.exec(k)[1];
        if (!monthToKeys.get(month)) {
            monthToKeys.set(month, []);
        }
        monthToKeys.get(month).push(k);
    }

    const rows = [["Month", "# of Users"]];
    const months = Array.from(monthToKeys.keys()).sort();
    for (const month of months) {
        rows.push([month, getNumUniqueGMs(monthToKeys.get(month))]);
    }

    const chart = new google.visualization.ColumnChart(element);
    chart.draw(google.visualization.arrayToDataTable(rows), {
        title: "# of Users Hosting Convocations",
        chartArea: {width: "50%"},
        hAxis: {
            title: "Month",
            showTextEvery: 3,
        },
        vAxis: {
            title: "# of Users",
            minValue: 0,
        },
    });
}

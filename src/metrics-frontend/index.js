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
    function getUniqueGMs(keys) {
        const gmAccountIds = new Set();
        for (const k of keys) {
            for (const convocation of convocations[k]) {
                for (const gmAccountId of convocation.admins) {
                    gmAccountIds.add(gmAccountId);
                }
            }
        }

        return gmAccountIds;
    }

    const monthToKeys = new Map();
    for (const k of Object.keys(convocations)) {
        const month = /^([0-9]{4}-[0-9]{2})-[0-9]{2}$/.exec(k)[1];
        if (!monthToKeys.get(month)) {
            monthToKeys.set(month, []);
        }
        monthToKeys.get(month).push(k);
    }

    const monthToUniqueGMs = new Map();
    for (const [month, keys] of monthToKeys.entries()) {
        monthToUniqueGMs.set(month, getUniqueGMs(keys));
    }

    const rows = [[
        "Month",
        "6+ Month Streak GMs",
        "5 Month Streak GMs",
        "4 Month Streak GMs",
        "3 Month Streak GMs",
        "2 Month Streak GMs",
        "No Streak GMs",
    ]];
    const months = Array.from(monthToUniqueGMs.keys()).sort();
    for (let i = 0; i < months.length; ++i) {
        const row = [months[i], 0, 0, 0, 0, 0, 0];
        const uniqueGMs = monthToUniqueGMs.get(months[i]);
        for (const gm of uniqueGMs) {
            let streakCount = 1;
            for (let j = i - 1; j > 0 && streakCount < 6; --j) {
                if (monthToUniqueGMs.get(months[j]).has(gm)) {
                    streakCount += 1;
                } else {
                    break;
                }
            }
            row[row.length - streakCount] += 1;
        }
        rows.push(row);
    }

    const chart = new google.visualization.ColumnChart(element);
    chart.draw(google.visualization.arrayToDataTable(rows), {
        title: "# of Users Hosting Convocations",
        chartArea: {
            width: "50%",
        },
        height: 800,
        hAxis: {
            title: "Month",
            showTextEvery: 3,
        },
        vAxis: {
            title: "# of Users",
            minValue: 0,
        },
        isStacked: true,
    });
}

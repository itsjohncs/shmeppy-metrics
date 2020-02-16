function main() {
    Promise.all([
        fetch("data/convocations.json")
            .then(function(r) {
                return r.json()
            }),
        fetch("data/registrations.json")
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
    ]).then(function([convocations, registrations]) {
        drawUniqueGMsPerMonth(
            document.getElementById("chart_uniqueGMsPerMonth"),
            convocations);
        drawRegistrations(
            document.getElementById("chart_registrations"),
            registrations);
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
        height: 500,
        hAxis: {
            title: "Month",
            showTextEvery: 3,
        },
        vAxis: {
            title: "# of Users",
        },
        isStacked: true,
    });
}


function drawRegistrations(element, registrations) {
    const monthsInOrder = Object.keys(registrations).sort();

    const rows = [["Month", "Total # of Registered Users", "# of Newly Registered Users that Day"]]
    let total = 0;
    for (const month of monthsInOrder) {
        const numNewUsers = registrations[month];
        total += numNewUsers;
        rows.push([new Date(month), total, numNewUsers]);
    }

    const chart = new google.visualization.LineChart(element);
    chart.draw(google.visualization.arrayToDataTable(rows), {
        title: "# of Registered Users",
        chartArea: {
            width: "50%",
        },
        height: 500,
        series: {
          0: {targetAxisIndex: 0},
          1: {targetAxisIndex: 1, lineWidth: 1, color: "#AAA"}
        },

        hAxis: {
            title: "Month",
            format: "yyyy-MM"
        },
        vAxes: [
            {
                title: "Users",
            },
            {
                title: "New Users that Day",
                gridlines: {count: 0},
                viewWindow: {max: 100},
            }
        ],
    });
}

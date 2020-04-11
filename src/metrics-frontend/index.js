function main() {
    Promise.all([
        fetch("data/convocations.json")
            .then(function(r) {
                return r.json()
            }),
        fetch("data/registrations.json?cachebust")
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
        drawUniqueUsersPerMonth(
            document.getElementById("chart_uniqueUsersPerMonth"),
            convocations);
        drawRegistrations(
            document.getElementById("chart_registrations"),
            registrations);
    });
}



function drawUniqueUsersPerMonth(element, convocations) {
    function getUniqueUsers(keys) {
        const accountIds = new Set();
        for (const k of keys) {
            for (const convocation of convocations[k]) {
                for (const gmAccountId of convocation.admins) {
                    accountIds.add(gmAccountId);
                }
                for (const playerAccountId of convocation.players) {
                    accountIds.add(playerAccountId);
                }
            }
        }

        return accountIds;
    }

    const monthToKeys = new Map();
    for (const k of Object.keys(convocations)) {
        const month = /^([0-9]{4}-[0-9]{2})-[0-9]{2}$/.exec(k)[1];
        if (!monthToKeys.get(month)) {
            monthToKeys.set(month, []);
        }
        monthToKeys.get(month).push(k);
    }

    const monthToUniqueAccountIds = new Map();
    for (const [month, keys] of monthToKeys.entries()) {
        monthToUniqueAccountIds.set(month, getUniqueUsers(keys));
    }

    const rows = [[
        "Month",
        "6+ Month Returning Users",
        "5 Month Returning Users",
        "4 Month Returning Users",
        "3 Month Returning Users",
        "2 Month Returning Users",
        "New Users",
    ]];
    const months = Array.from(monthToUniqueAccountIds.keys()).sort();
    const runningReturnCounts = new Map();
    function incrementReturnCount(accountId) {
        if (runningReturnCounts.has(accountId)) {
            const newCount = runningReturnCounts.get(accountId) + 1;
            runningReturnCounts.set(accountId, newCount);
            return newCount;
        } else {
            runningReturnCounts.set(accountId, 1);
            return 1;
        }
    }
    for (let i = 0; i < months.length; ++i) {
        const row = [months[i], 0, 0, 0, 0, 0, 0];
        const uniqueAccountIds = monthToUniqueAccountIds.get(months[i]);
        for (const accountId of uniqueAccountIds) {
            const count = Math.min(incrementReturnCount(accountId), 6);
            row[row.length - count] += 1;
        }
        rows.push(row);
    }

    const chart = new google.visualization.ColumnChart(element);
    chart.draw(google.visualization.arrayToDataTable(rows), {
        title: "# of Users in Convocations (Players & Admins)",
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
        "6+ Month Returning GMs",
        "5 Month Returning GMs",
        "4 Month Returning GMs",
        "3 Month Returning GMs",
        "2 Month Returning GMs",
        "New GMs",
    ]];
    const months = Array.from(monthToUniqueGMs.keys()).sort();
    const runningGmReturnCounts = new Map();
    function incrementReturnCount(gm) {
        if (runningGmReturnCounts.has(gm)) {
            const newCount = runningGmReturnCounts.get(gm) + 1;
            runningGmReturnCounts.set(gm, newCount);
            return newCount;
        } else {
            runningGmReturnCounts.set(gm, 1);
            return 1;
        }
    }
    for (let i = 0; i < months.length; ++i) {
        const row = [months[i], 0, 0, 0, 0, 0, 0];
        const uniqueGMs = monthToUniqueGMs.get(months[i]);
        for (const gm of uniqueGMs) {
            const count = Math.min(incrementReturnCount(gm), 6);
            row[row.length - count] += 1;
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

    const rows = [[
        "Month",
        "Total # of Registered Users",
        "# of Newly Registered Users that Day",
    ]];
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
            }
        ],
    });
}

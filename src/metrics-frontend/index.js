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
        fetch("data/active-users.json?ham")
            .then(function(r) {
                return r.json()
            }),
        fetch("data/event-counts.json")
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
    ]).then(function([convocations, registrations, activeUsers, eventCounts]) {
        drawUniqueGMsPerMonth(
            document.getElementById("chart_uniqueGMsPerMonth"),
            convocations);
        drawUniqueUsersPerMonth(
            document.getElementById("chart_uniqueUsersPerMonth"),
            convocations);
        drawRegistrations(
            document.getElementById("chart_registrations"),
            registrations);

        function hours(n) {
            return n * 60 * 60;
        }

        drawActiveUsersWindow({
            element: document.getElementById("chart_activeUsers"),
            usersByDay: activeUsers,
            daysInWindow: 30,
            buckets: [
                [hours(24), "24+ hours"],
                [hours(12), "12+ hours"],
                [hours(6), "6+ hours"],
                [hours(3), "3+ hours"],
                [hours(1), "1+ hours"],
                [0, "<1 hour"],
            ],
        });
        drawActiveUsersWindow({
            element: document.getElementById("chart_activeUsersWindow"),
            usersByDay: activeUsers,
            daysInWindow: 7,
            buckets: [
                [hours(12), "12+ hours"],
                [hours(6), "6+ hours"],
                [hours(3), "3+ hours"],
                [hours(2), "2+ hours"],
                [hours(1), "1+ hours"],
                [0, "<1 hour"],
            ]
        });

        drawEventCounts(
            document.getElementById("chart_eventCounts"),
            eventCounts);
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


function drawActiveUsersWindow({element, usersByDay, daysInWindow, buckets}) {
    const rows = [[
        {label: "Month", type: "date"},
        ...buckets.map(function([_, label]) {
            return {label, type: "number"};
        }),
    ]];

    function hours(n) {
        return n * 60 * 60;
    }

    const thresholds = [
        Number.MAX_VALUE,
        ...buckets.map(function([threshold, _]) {
            return threshold;
        }),
    ];
    function getRowIndex(seconds) {
        for (let i = thresholds.length - 2; i >= 0; --i) {
            if (seconds < thresholds[i]) {
                return i + 1;
            }
        }
    }

    for (const anchorDay of Object.keys(usersByDay)) {
        // No month before this really had any usage
        if (new Date(anchorDay) < new Date("2019-06-01")) {
            continue;
        }

        const tooRecently = new Date();
        tooRecently.setDate(tooRecently.getDate() - 2);
        if (new Date(anchorDay) >= tooRecently) {
            continue;
        }

        const endDay = new Date(anchorDay);
        const startDay = new Date(anchorDay);
        startDay.setDate(startDay.getDate() - daysInWindow);
        const row = [new Date(anchorDay), 0, 0, 0, 0, 0, 0];

        const usersToTotalTime = new Map();
        for (const [day, users] of Object.entries(usersByDay)) {
            const dt = new Date(day);
            if (dt < startDay || dt > endDay) {
                continue;
            }

            for (const [accountId, seconds] of Object.entries(users)) {
                if (usersToTotalTime.has(accountId)) {
                    usersToTotalTime.set(
                        accountId, usersToTotalTime.get(accountId) + seconds);
                } else {
                    usersToTotalTime.set(accountId, seconds);
                }
            }
        }

        for (const seconds of usersToTotalTime.values()) {
            row[getRowIndex(seconds)] += 1;
        }
        rows.push(row);
    }

    rows.sort(function(a, b) {
        const aDt = new Date(a[0]);
        const bDt = new Date(b[0]);
        if (aDt < bDt) {
            return -1;
        } else if (aDt > bDt) {
            return 1;
        } else {
            return 0;
        }
    });

    const chart = new google.visualization.SteppedAreaChart(element);
    chart.draw(google.visualization.arrayToDataTable(rows), {
        title: `# of GMs in Any Games (during Game Activity), ${daysInWindow}-day window`,
        chartArea: {
            width: "50%",
        },
        height: 500,
        hAxis: {
            title: "Date",
        },
        vAxis: {
            title: "# of GMs",
        },
        isStacked: true,
    });
}


function drawEventCounts(element, eventCounts) {
    const rows = [[
        {label: "Day", type: "date"},
        ...Object.entries(eventCounts).map(function([eventName, _]) {
            return {label: eventName, type: "number"};
        }),
    ]];

    const allRawDates = new Set();
    for (const eventData of Object.values(eventCounts)) {
        for (const rawDate of Object.keys(eventData)) {
            allRawDates.add(rawDate);
        }
    }

    const sortedRawDates = Array.from(allRawDates).sort(function(a, b) {
        const aDt = new Date(a);
        const bDt = new Date(b);
        if (aDt < bDt) {
            return -1;
        } else if (aDt > bDt) {
            return 1;
        } else {
            return 0;
        }
    });

    for (const rawDate of sortedRawDates) {
        const date = new Date(rawDate);
        if (date < new Date("2019-06-01")) {
            continue;
        }
        const row = [date];

        // I use Object.entries here (and above when declaring the rows)
        // because I'm being overly paranoid and worrying that Object.values
        // an Object.keys could iterate in slightly different orderings
        // sometimes... (even though that's probably not true)
        for (const [_, eventData] of Object.entries(eventCounts)) {
            row.push(eventData[rawDate] || 0);
        }

        rows.push(row);
    }

    console.log(rows);

    const chart = new google.visualization.LineChart(element);
    chart.draw(google.visualization.arrayToDataTable(rows), {
        title: "Event Counts",
        chartArea: {
            width: "50%",
        },
        height: 500,
        hAxis: {
            title: "Date",
        },
        vAxis: {
            title: "Count",
        },
    });
}

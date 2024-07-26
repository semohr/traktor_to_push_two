import CSI 1.0
import QtQuick 2.0

import "ApiClient.js" as ApiClient

Item {

    AppProperty { path: "app.traktor.decks.1.is_loaded";               onValueChanged: testapifxtimer.start() }
    AppProperty {id: allstuff; path: "app.traktor.fx.1.dry_wet"}



    Timer {
        id: testapifxtimer
        repeat: true
        interval: 250

        onTriggered: onFxChanged()
    }

    Component.onCompleted: {
        onFxChanged();
    }

    function sendAppTraktorProperties() {
        var traktor = allstuff;  // Assuming `allstuff` directly refers to the object you want to inspect
        var properties = Object.keys(traktor);
        for (var i = 0; i < properties.length; i++) {
            try {
                var value = traktor[properties[i]];
                // Test if the property is serializable by attempting to stringify it
                JSON.stringify(value);
                propertiesObject[properties[i]] = value;
            } catch (e) {
                // Property is not serializable, skip it
                console.warn("Skipping non-serializable property: " + properties[i]);
            }
        }
        ApiClient.send("appTraktorProperties", propertiesObject);
    }

    function onFxChanged(){
        

        ApiClient.send("fxUnitType", {
            all: "foo",
            stuff: allstuff
        }) 
    }
}
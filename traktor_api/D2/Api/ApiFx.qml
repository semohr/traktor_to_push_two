import CSI 1.0
import QtQuick 2.0
import "ApiClient.js" as ApiClient

Item {    
    property int fxUnitId: 1;

    AppProperty { id: fxUnitType; path: `app.traktor.fx.${fxUnitId}.type`; onValueChanged: onFxChanged("Type",fxUnitType)}
    AppProperty { id: fxSelect1; path: `app.traktor.fx.${fxUnitId}.select.1`; onValueChanged: onFxChanged("Select", fxSelect1)}
    AppProperty { id: fxSelect2; path: `app.traktor.fx.${fxUnitId}.select.2`; onValueChanged: onFxChanged("Select", fxSelect2)}
    AppProperty { id: fxSelect3; path: `app.traktor.fx.${fxUnitId}.select.3`; onValueChanged: onFxChanged("Select", fxSelect3)}


    AppProperty { id: fxDryWet; path: `app.traktor.fx.${fxUnitId}.dry_wet`; onValueChanged: onFxChanged("DryWet",fxDryWet)}

    // Knobs
    AppProperty { id: fxKnob1; path: `app.traktor.fx.${fxUnitId}.knobs.1`; onValueChanged: onFxChanged("Knob",fxKnob1)}
    AppProperty { id: fxKnob2; path: `app.traktor.fx.${fxUnitId}.knobs.2`; onValueChanged: onFxChanged("Knob",fxKnob2)}
    AppProperty { id: fxKnob3; path: `app.traktor.fx.${fxUnitId}.knobs.3`; onValueChanged: onFxChanged("Knob",fxKnob3)}

    AppProperty { id: fxKnobName1; path: `app.traktor.fx.${fxUnitId}.knobs.1.name`;   onValueChanged: onFxChanged("Name",fxKnobName1)}
    AppProperty { id: fxKnobName2; path: `app.traktor.fx.${fxUnitId}.knobs.2.name`;   onValueChanged: onFxChanged("Name",fxKnobName2)}
    AppProperty { id: fxKnobName3; path: `app.traktor.fx.${fxUnitId}.knobs.3.name`;   onValueChanged: onFxChanged("Name",fxKnobName3)}

    AppProperty { id: fxParameterValue1; path: `app.traktor.fx.${fxUnitId}.parameters.1`; onValueChanged: onFxChanged("Param",fxParameterValue1)}
    AppProperty { id: fxParameterValue2; path: `app.traktor.fx.${fxUnitId}.parameters.2`; onValueChanged: onFxChanged("Param",fxParameterValue2)}
    AppProperty { id: fxParameterValue3; path: `app.traktor.fx.${fxUnitId}.parameters.3`; onValueChanged: onFxChanged("Param",fxParameterValue3)}

    function onFxChanged(type, event){
        const d = {};
        d[type] = event;
        ApiClient.send("fx/"+fxUnitId,d)
    }


  Timer {
    id: fxtimerunit
    repeat: true
    interval: 10000
    running: true

    onTriggered: {
            // Trigger onFxChanged for each property
            onFxChanged("Type", fxUnitType);
            onFxChanged("Select", fxSelect1);
            onFxChanged("Select", fxSelect2);
            onFxChanged("Select", fxSelect3);
            onFxChanged("DryWet", fxDryWet);
            onFxChanged("Knob", fxKnob1);
            onFxChanged("Knob", fxKnob2);
            onFxChanged("Knob", fxKnob3);
            onFxChanged("Name", fxKnobName1);
            onFxChanged("Name", fxKnobName2);
            onFxChanged("Name", fxKnobName3);
            onFxChanged("Param", fxParameterValue1);
            onFxChanged("Param", fxParameterValue2);
            onFxChanged("Param", fxParameterValue3);
    }
  }
}


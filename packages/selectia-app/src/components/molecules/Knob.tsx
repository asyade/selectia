import { useState } from 'react'
import { Donut } from 'react-dial-knob'

export default function Knob() {
    const [value, setValue] = useState(0)
    return <Donut
        style={{
            textAlign: 'center',
        }}
        diameter={48}
        min={0}
        max={100}
        step={1}
        value={value}
        theme={{
            bgrColor: '#1e1e1e',
            donutColor: '#7c3aed',
            centerColor: '#21222c',
            centerFocusedColor: '#21222c',
            donutThickness: 4,
        }}
        onValueChange={setValue}
        ariaLabelledBy={'my-label'}
    >
        <label className="text-primary text-center text-xs w-full" id={'my-label'}>Volume</label>
    </Donut>
}

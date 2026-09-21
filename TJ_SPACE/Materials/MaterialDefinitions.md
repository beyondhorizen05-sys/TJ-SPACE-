# System 1 master materials

Every material exposes scalar parameter DegradedStrained in range 0..1, default 0.0. It is connected to bounded visual stress.

- M_Metal_Brushed: metallic 1.0, roughness 0.28 -> 0.62.
- M_Glass_Smart: translucent, roughness 0.08 -> 0.38; reduced clarity under stress.
- M_Concrete_Sovereign: roughness 0.78 -> 0.92; bounded darkening under stress.
- M_Emissive_Circuit: emissive 8.0 -> 2.0; increased trace noise under stress.
- M_Holo_Panel: opacity 0.42 -> 0.58; emission 6.0 -> 1.5; scanline breakup under stress.
- M_Data_Flow: emission 12.0 -> 3.0; pulse irregularity under stress.
- M_Shield_TLS: opacity 0.20 -> 0.36; emission 3.5 -> 1.0; edge noise under stress.
- M_Energy_Conduit: core emission 10.0 -> 2.5; sheath roughness rises under stress.

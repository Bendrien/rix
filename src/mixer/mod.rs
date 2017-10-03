const INPUTS: usize = 3;
const OUTPUTS: usize = 2;
const BUFFER_SIZE: usize = 1;

type Sample = f32;

#[derive(Clone, Debug)]
pub enum Buffer {
    None,
    Mono([Sample; BUFFER_SIZE]),
    Stereo {
        l: [Sample; BUFFER_SIZE],
        r: [Sample; BUFFER_SIZE],
    },
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer::None
    }
}

#[derive(Clone, Debug)]
struct MixProperty {
    gain: f32,
    pan: f32,
    mute: bool,
}

impl Default for MixProperty {
    fn default() -> Self {
        Self {
            gain: 1.,
            pan: 0.5,
            mute: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Link {
    active: bool,
    property: MixProperty,
}

impl Default for Link {
    fn default() -> Self {
        Self {
            active: true,
            property: MixProperty::default(),
        }
    }
}

#[derive(Debug)]
pub struct Mixer {
    inputs:  [Buffer; INPUTS],
    outputs: [Buffer; OUTPUTS],
    pre:     [MixProperty; INPUTS],
    post:    [MixProperty; OUTPUTS],
    intern:  [Link; INPUTS * OUTPUTS],
    // TODO: external/automation get their own link-matrix
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            inputs:  Default::default(),
            outputs: Default::default(),
            pre:     Default::default(),
            post:    Default::default(),
            intern:  Default::default(),
        }
    }

    pub fn process(&mut self) {
        process_matrix(&self.intern, &self.inputs, &mut self.outputs).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixError {
    DimensionMismatch
}

fn process_matrix(matrix: &[Link], inputs: &[Buffer], outputs: &mut [Buffer]) -> Result<(), MatrixError> {
    if matrix.len() != inputs.len() * outputs.len() {
        return Err(MatrixError::DimensionMismatch);
    }

    for ((links, inputs), outBuf) in matrix.chunks(inputs.len())
        .zip(inputs.chunks(inputs.len()).cycle())
        .zip(outputs.iter_mut()) {
        for (link, inBuf) in links.iter()
            .zip(inputs.iter())
            .filter(|&(link, _)| link.active && !link.property.mute) {
            use self::Buffer::{None, Mono, Stereo};
            use std::f32::consts::PI;

            let MixProperty { gain: gain, pan: pan, mute: mute, .. } = link.property;
            match &outBuf {
                &&mut None => (),
                &&mut Mono(mut outBuf) =>
                    match inBuf {
                        &None =>
                            for out in outBuf.iter_mut() {
                                // scale the pan in to bipolar domain
                                let scale = 2. * pan - 1.;
                                *out += gain * scale;
                            },
                        &Mono(inBuf) =>
                            for (out, inM) in outBuf.iter_mut()
                                .zip(inBuf.iter()) {
                                *out += inM * gain;
                            },
                        &Stereo { l: inBufL, r: inBufR } =>
                            for (out, (inL, inR)) in outBuf.iter_mut()
                                .zip(inBufL.iter()
                                    .zip(inBufR.iter())) {
                                let rad = pan * 0.5 * PI;
                                *out += (inL * rad.cos() + inR * rad.sin()) * gain;
                            },
                        _ => unimplemented!(),
                    },
                &&mut Stereo { l: mut outBufL, r: mut outBufR } =>
                    match inBuf {
                        &None =>
                            for (outL, outR) in outBufL.iter_mut()
                                .zip(outBufR.iter_mut()) {
                                // equal power panning
                                let rad = pan * 0.5 * PI;
                                // scale the gain in to bipolar domain
                                let bipolar = 2. * gain - 1.;
                                *outL += rad.cos() * bipolar;
                                *outR += rad.sin() * bipolar;
                            },
                        &Mono(inBuf) =>
                            for ((outL, outR), inM) in outBufL.iter_mut()
                                .zip(outBufR.iter_mut())
                                .zip(inBuf.iter()) {
                                // equal power panning
                                let rad = pan * 0.5 * PI;
                                *outL += inM * rad.cos() * gain;
                                *outR += inM * rad.sin() * gain;
                            },
                        &Stereo { l: inBufL, r: inBufR } =>
                            for ((outL, outR), (inL, inR)) in outBufL.iter_mut()
                                .zip(outBufR.iter_mut())
                                .zip(inBufL.iter()
                                    .zip(inBufR.iter())) {
                                match (pan, inL == inR) {
                                    (pan, false) if pan < 0.5 => {
                                        // stereo separation
                                        let ratio = pan * PI;
                                        *outL = (inL + inR * ratio) * gain;
                                        *outR = (inR + inL * ratio) * gain;
                                    },
                                    (pan, false) if pan > 0.5 => {
                                        // merging
                                        let ratio1 = ((1.5 - pan) * 0.5 * PI).cos();
                                        let ratio2 = ((pan - 0.5) * 0.5 * PI).sin();
                                        *outL = (inL * ratio1 + inR * ratio2) * gain;
                                        *outR = (inR * ratio1 + inL * ratio2) * gain;
                                    },
                                    _ => {
                                        *outL += inL * gain;
                                        *outR += inR * gain;
                                    }
                                }
                            },
                        _ => unimplemented!(),
                    },
                _ => unimplemented!(),
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut mxr = Mixer::new();
        mxr.process();
        println!("{:?}", mxr);
    }
}
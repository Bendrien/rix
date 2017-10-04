use arraydeque::ArrayDeque;

pub const BUFFER_SIZE: usize = 32;
pub const INPUTS: usize = 4;
pub const OUTPUTS: usize = 8;

type Sample = f32;

#[derive(Clone, Debug)]
pub enum Buffer {
    None,
    Mono(ArrayDeque<[Sample; BUFFER_SIZE]>),
    Stereo {
        l: ArrayDeque<[Sample; BUFFER_SIZE]>,
        r: ArrayDeque<[Sample; BUFFER_SIZE]>,
    },
}

impl Default for Buffer {
    fn default() -> Self {
        //Buffer::None
        Buffer::Mono(Default::default())
        //Buffer::Stereo{l:Default::default(),r:Default::default()}
    }
}

impl Buffer {
    pub fn clear(&mut self) {
        use self::Buffer::{None, Mono, Stereo};

        match self {
            &mut None => (),
            &mut Mono(ref mut buf) => buf.clear(),
            &mut Stereo{ ref mut l, ref mut r } => {
                l.clear();
                r.clear();
            }
            _ => unimplemented!(),
        }
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
            mute: true,
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

#[derive(Default, Debug)]
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

pub fn process_matrix(matrix: &[Link], inputs: &[Buffer], outputs: &mut [Buffer]) -> Result<(), MatrixError> {
    if matrix.len() != inputs.len() * outputs.len() {
        return Err(MatrixError::DimensionMismatch);
    }

    for ((links, inputs), out_buf) in matrix.chunks(inputs.len())
        .zip(inputs.chunks(inputs.len()).cycle())
        .zip(outputs.iter_mut()) {
        for (link, in_buf) in links.iter()
            .zip(inputs.iter())
            .filter(|&(link, _)| link.active && !link.property.mute) {
            use core::f32::consts::PI;
            use self::Buffer::{None, Mono, Stereo};

            let MixProperty { gain, pan, .. } = link.property;
            match out_buf {
                &mut None => (),
                &mut Mono(ref mut out_buf) =>
                    match in_buf {
                        &None =>
                            for out in out_buf.iter_mut() {
                                // scale the pan in to bipolar domain
                                let scale = 2. * pan - 1.;
                                *out += gain * scale;
                            },
                        &Mono(ref in_buf) =>
                            for (out, in_m) in out_buf.iter_mut()
                                .zip(in_buf.iter()) {
                                *out += in_m * gain;
                            },
                        &Stereo { l: ref in_buf_l, r: ref in_buf_r } =>
                            for (out, (in_l, in_r)) in out_buf.iter_mut()
                                .zip(in_buf_l.iter()
                                    .zip(in_buf_r.iter())) {
                                let rad = pan * 0.5 * PI;
                                *out += (in_l * rad.cos() + in_r * rad.sin()) * gain;
                            },
                        _ => unimplemented!(),
                    },
                &mut Stereo { l: ref mut out_buf_l, r: ref mut out_buf_r } =>
                    match in_buf {
                        &None =>
                            for (out_l, out_r) in out_buf_l.iter_mut()
                                .zip(out_buf_r.iter_mut()) {
                                // equal power panning
                                let rad = pan * 0.5 * PI;
                                // scale the gain in to bipolar domain
                                let bipolar = 2. * gain - 1.;
                                *out_l += rad.cos() * bipolar;
                                *out_r += rad.sin() * bipolar;
                            },
                        &Mono(ref in_buf) =>
                            for ((out_l, out_r), in_m) in out_buf_l.iter_mut()
                                .zip(out_buf_r.iter_mut())
                                .zip(in_buf.iter()) {
                                // equal power panning
                                let rad = pan * 0.5 * PI;
                                *out_l += in_m * rad.cos() * gain;
                                *out_r += in_m * rad.sin() * gain;
                            },
                        &Stereo { l: ref in_buf_l, r: ref in_buf_r } =>
                            for ((out_l, out_r), (in_l, in_r)) in out_buf_l.iter_mut()
                                .zip(out_buf_r.iter_mut())
                                .zip(in_buf_l.iter()
                                    .zip(in_buf_r.iter())) {
                                match (pan, in_l == in_r) {
                                    (pan, false) if pan < 0.5 => {
                                        // stereo separation
                                        let ratio = pan * PI;
                                        *out_l = (in_l + in_r * ratio) * gain;
                                        *out_r = (in_r + in_l * ratio) * gain;
                                    },
                                    (pan, false) if pan > 0.5 => {
                                        // merging
                                        let ratio1 = ((1.5 - pan) * 0.5 * PI).cos();
                                        let ratio2 = ((pan - 0.5) * 0.5 * PI).sin();
                                        *out_l = (in_l * ratio1 + in_r * ratio2) * gain;
                                        *out_r = (in_r * ratio1 + in_l * ratio2) * gain;
                                    },
                                    _ => {
                                        *out_l += in_l * gain;
                                        *out_r += in_r * gain;
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
        println!("{:#?}", mxr);
    }
}
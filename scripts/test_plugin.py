import numpy as np
from pedalboard import load_plugin

PLUGIN_PATH = 'target/bundled/Subrou Rs.vst3'


def run():
    plugin = load_plugin(PLUGIN_PATH)
    buf = np.zeros((2, 256), dtype=np.float32)
    out = plugin(buf, sample_rate=44100)
    assert np.allclose(out, 0.0)

    buf.fill(1.0)
    out = plugin(buf, sample_rate=44100)
    assert out.shape == buf.shape
    assert not np.allclose(out, buf)

    high_sr = np.ones((2, 128), dtype=np.float32)
    out = plugin(high_sr, sample_rate=48000)
    assert out.shape == high_sr.shape

if __name__ == '__main__':
    run()
    print('Python plugin tests passed')

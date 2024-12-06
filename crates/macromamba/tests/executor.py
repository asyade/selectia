import demucs.api
import torch

print(torch.cuda.get_arch_list())
device = "cuda" if torch.cuda.is_available() else "cpu"
print(f"Using device: {device}")

separator = demucs.api.Separator()
separator = demucs.api.Separator(model="mdx_extra", segment=128)
origin, separated = separator.separate_audio_file("./tests/128-bpm.wav")
# origin, separated = separator.separate_tensor(audio)
# separator.update_parameter(segment=smaller_segment)


# for file, sources in separated:
for stem, source in separated.items():
    file = "128-bpm.wav"
    demucs.api.save_audio(source, f"{stem}_{file}", samplerate=separator.samplerate)

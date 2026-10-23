import random, os
random.seed(42)
def w(name, s, nl=None):
    with open('data/'+name, 'w', encoding='utf-8', newline='') as f:
        f.write(s)
w('ascii.txt', "The quick brown fox jumps over the lazy dog.\nThe dog barks; the fox runs away!\n\nA fox is a fox, a dog is a dog.\nthe END\n")
w('unicode.txt', "Привет мир! Привет, мир.\nΓειά σου κόσμε γειά σου\n日本語 のテキスト 世界 世界\nمرحبا بالعالم مرحبا\nहिंदी पाठ नमस्ते नमस्ते\n😀 😀 😃 🎉 👨‍👩‍👧‍👦 café café ÉCOLE école\nＡＢＣ \U0001F600 ﬃ straße STRASSE\n")
w('empty.txt', "")
w('crlf.txt', "line one\r\nline two has more words\r\n\r\nline four\r\nlast line no newline")
w('longline.txt', " ".join(random.choice(["alpha","beta","gamma","delta","epsilon","zeta"]) for _ in range(40000)))
vocab = ["lorem","ipsum","dolor","sit","amet","consectetur","adipiscing","elit","sed","do","eiusmod","tempor","incididunt","ut","labore","et","dolore","magna","aliqua","enim","ad","minim","veniam","quis","nostrud","exercitation","ullamco","laboris","nisi","aliquip","ex","ea","commodo","consequat","Λόγος","世界","привет","😀"]
parts=[]; size=0
while size < 2*1024*1024:
    line = " ".join(random.choice(vocab) for _ in range(random.randint(3,14)))
    parts.append(line); size += len(line.encode('utf-8'))+1
w('big.txt', "\n".join(parts)+"\n")
w('nofinal.txt', "one two\nthree")
os.makedirs('data/sub', exist_ok=True)
w('sub/nested.txt', "nested file here\n")
w('notes.md', "not a txt file\n")
print({n: os.path.getsize('data/'+n) for n in os.listdir('data') if os.path.isfile('data/'+n)})

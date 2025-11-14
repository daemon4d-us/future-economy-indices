"""Quick test of Anthropic API key."""

import os
from dotenv import load_dotenv
from anthropic import Anthropic

load_dotenv()

api_key = os.getenv("ANTHROPIC_API_KEY")
print(f"API Key found: {api_key[:20]}...")

client = Anthropic(api_key=api_key)

# Try different model names
models_to_try = [
    "claude-3-5-sonnet-20241022",
    "claude-3-5-sonnet-20240620",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
    "claude-3-opus-20240229",
]

for model in models_to_try:
    try:
        print(f"\nTrying model: {model}")
        response = client.messages.create(
            model=model,
            max_tokens=100,
            messages=[{"role": "user", "content": "Say 'test successful'"}]
        )
        print(f"✓ SUCCESS with {model}")
        print(f"Response: {response.content[0].text}")
        break
    except Exception as e:
        print(f"✗ Failed: {e}")

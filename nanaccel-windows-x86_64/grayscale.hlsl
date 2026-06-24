Texture2D shaderTexture : register(t0);
SamplerState SampleType : register(s0);

struct VS_OUTPUT {
    float4 Pos : SV_POSITION;
    float2 Tex : TEXCOORD0;
};

float4 main(VS_OUTPUT input) : SV_TARGET {
    // Sample color from decoded texture
    float4 color = shaderTexture.Sample(SampleType, input.Tex);
    
    // Calculate grayscale using standard coefficients
    float gray = dot(color.rgb, float3(0.299, 0.587, 0.114));
    
    return float4(gray, gray, gray, color.a);
}

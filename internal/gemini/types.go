package gemini

// Request types for Gemini generateContent API.

type GenerateRequest struct {
	Contents         []Content        `json:"contents"`
	GenerationConfig GenerationConfig `json:"generationConfig"`
}

type Content struct {
	Role  string `json:"role"`
	Parts []Part `json:"parts"`
}

type Part struct {
	Text       string      `json:"text,omitempty"`
	InlineData *InlineData `json:"inlineData,omitempty"`
}

type InlineData struct {
	MIMEType string `json:"mimeType"`
	Data     string `json:"data"`
}

type GenerationConfig struct {
	ResponseModalities []string `json:"responseModalities"`
	// ImageConfig carries aspectRatio/imageSize. omitempty keeps a request with no
	// aspect/resolution byte-identical to the pre-imageConfig shape (verified: a bare
	// call returns 200 unchanged — see findings/exp-001-model-schema.md).
	ImageConfig *ImageConfig `json:"imageConfig,omitempty"`
}

// ImageConfig is the generationConfig.imageConfig block (Schema A, live-verified).
// The official-docs "responseFormat.image" paraphrase is wrong; this is the shape the
// API accepts. The API silently ignores invalid enum values, so callers MUST validate
// aspectRatio/imageSize client-side before sending (see ValidateImageConfig).
type ImageConfig struct {
	AspectRatio string `json:"aspectRatio,omitempty"`
	ImageSize   string `json:"imageSize,omitempty"`
}

// Response types for Gemini generateContent API.

type GenerateResponse struct {
	Candidates     []Candidate     `json:"candidates"`
	PromptFeedback *PromptFeedback `json:"promptFeedback,omitempty"`
}

type Candidate struct {
	Content       *Content       `json:"content"`
	FinishReason  string         `json:"finishReason,omitempty"`
	SafetyRatings []SafetyRating `json:"safetyRatings,omitempty"`
}

type SafetyRating struct {
	Category    string `json:"category"`
	Probability string `json:"probability"`
}

type PromptFeedback struct {
	BlockReason   string         `json:"blockReason,omitempty"`
	SafetyRatings []SafetyRating `json:"safetyRatings,omitempty"`
}

// ImageResult holds a decoded image from the API response.
type ImageResult struct {
	Data     []byte
	MIMEType string
}

// ErrorResponse represents a Gemini API error.
type ErrorResponse struct {
	Error struct {
		Code    int    `json:"code"`
		Message string `json:"message"`
		Status  string `json:"status"`
	} `json:"error"`
}

package gemini

import (
	"fmt"
	"strings"
)

// ValidAspectRatios and ValidImageSizes are the enum values the API accepts for
// generationConfig.imageConfig. The API silently ignores invalid values (returns a
// default-size image with HTTP 200 — see findings/exp-001-model-schema.md), so naba
// MUST validate client-side; otherwise users get silently-wrong output.
var (
	ValidAspectRatios = []string{
		"1:1", "1:4", "1:8", "2:3", "3:2", "3:4", "4:1", "4:3",
		"4:5", "5:4", "8:1", "9:16", "16:9", "21:9",
	}
	// ValidImageSizes uses uppercase K (512, 1K, 2K, 4K). A lowercase "1k" is rejected
	// because the API would accept it as HTTP 200 yet ignore it.
	ValidImageSizes = []string{"512", "1K", "2K", "4K"}
)

// NewImageConfig builds a validated *ImageConfig from aspect and resolution strings.
// Empty inputs are omitted; when both are empty it returns (nil, nil) so the request
// stays byte-identical to a bare call. An invalid value yields an *APIError with
// ExitUsage and a message listing the accepted values.
func NewImageConfig(aspect, resolution string) (*ImageConfig, error) {
	if aspect == "" && resolution == "" {
		return nil, nil
	}
	if aspect != "" && !contains(ValidAspectRatios, aspect) {
		return nil, &APIError{
			ExitCode: ExitUsage,
			Message: fmt.Sprintf("invalid aspect ratio %q\n\nValid values: %s",
				aspect, strings.Join(ValidAspectRatios, ", ")),
		}
	}
	if resolution != "" && !contains(ValidImageSizes, resolution) {
		return nil, &APIError{
			ExitCode: ExitUsage,
			Message: fmt.Sprintf("invalid resolution %q\n\nValid values: %s",
				resolution, strings.Join(ValidImageSizes, ", ")),
		}
	}
	return &ImageConfig{AspectRatio: aspect, ImageSize: resolution}, nil
}

// ModelForQuality maps a --quality alias to a concrete model id: fast→flash, high→pro.
// An unrecognized value yields an ExitUsage error. This is the only quality→model
// mapping; the config `quality` key (internal/config) resolves through here too.
func ModelForQuality(quality string) (string, error) {
	switch quality {
	case "fast":
		return FlashModel, nil
	case "high":
		return ProModel, nil
	default:
		return "", &APIError{
			ExitCode: ExitUsage,
			Message:  fmt.Sprintf("invalid quality %q\n\nValid values: fast, high", quality),
		}
	}
}

func contains(set []string, v string) bool {
	for _, s := range set {
		if s == v {
			return true
		}
	}
	return false
}

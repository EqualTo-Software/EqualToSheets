import { FormEventHandler, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import styled from 'styled-components/macro';
import { SSOButton, SubmitButton } from '../../components/buttons';
import { EmailInput } from '../../components/emailInput';
import { DualBox, ExternalLink, HeadingText, LeftSide, RightSide, Subtitle } from './common';
import { ReactComponent as GitHubSSOIcon } from './icons/github.svg';
import { ReactComponent as GoogleSSOIcon } from './icons/google.svg';
import { ReactComponent as MicrosoftSSOIcon } from './icons/microsoft.svg';
import { VideoEmbed } from './videoEmbed';

export const LicenseRequestPage = () => {
  const navigate = useNavigate();

  const [isDuringSubmit, setDuringSubmit] = useState(false);
  const [email, setEmail] = useState('');
  const [emailError, setEmailError] = useState(false);
  const [requestError, setRequestError] = useState<string | null>(null);

  const onSubmit: FormEventHandler<HTMLFormElement> = (event) => {
    event.preventDefault();

    sendRequest();

    async function sendRequest() {
      const simpleEmailRegex = /^[^@\s]+@[^@\s]+$/;
      const sanitizedMail = email.trim();

      if (!simpleEmailRegex.test(sanitizedMail)) {
        setEmailError(true);
        return;
      }

      setDuringSubmit(true);
      setRequestError(null);
      setEmailError(false);

      const formData = new FormData();
      formData.append('email', sanitizedMail);

      let response;
      try {
        response = await fetch('./send-license-key', {
          method: 'POST',
          headers: {
            Accept: 'application/json',
          },
          body: formData,
        });
      } catch {
        setRequestError('Could not connect to a server. Please try again.');
        setDuringSubmit(false);
        return;
      }

      if (!response.ok) {
        setRequestError(await response.text());
        setDuringSubmit(false);
        return;
      }

      setDuringSubmit(false);
      navigate('/license/sent');
    }
  };

  return (
    <DualBox>
      <LeftSide>
        <HeadingText>
          Join the <em>EqualTo Sheets</em> open beta
        </HeadingText>
        <Subtitle>Integrate a high-performance spreadsheet in minutes</Subtitle>
        <VideoEmbed />
        <Subtitle>
          {'Learn more from '}
          <ExternalLink target="_blank" href="https://docs.equalto.com">
            our docs
          </ExternalLink>
        </Subtitle>
      </LeftSide>
      <RightSide>
        <div />
        <Form onSubmit={onSubmit}>
          <EmailInput
            disabled={isDuringSubmit}
            name="email"
            onChange={(event) => setEmail(event.target.value)}
            error={emailError}
            autoFocus
          />
          <SubmitButton disabled={isDuringSubmit} type="submit">
            Get access now
          </SubmitButton>
          {requestError && <ErrorMessage>{requestError}</ErrorMessage>}
        </Form>
        <SSO>
          <SSOButton
            startIcon={
              <SSOIcon>
                <GoogleSSOIcon />
              </SSOIcon>
            }
            onClick={() => {
              window.location.assign(getSSOLoginURL('google'));
            }}
          >
            Log in with Google
          </SSOButton>
          <SSOButton
            startIcon={
              <SSOIcon>
                <GitHubSSOIcon />
              </SSOIcon>
            }
            onClick={() => {
              window.location.assign(getSSOLoginURL('github'));
            }}
          >
            Log in with GitHub
          </SSOButton>
          <SSOButton
            startIcon={
              <SSOIcon>
                <MicrosoftSSOIcon />
              </SSOIcon>
            }
            onClick={() => {
              window.location.assign(getSSOLoginURL('microsoft'));
            }}
          >
            Log in with Microsoft
          </SSOButton>
        </SSO>
        <FormFooterText>
          {'By submitting my details, I agree to the '}
          <ExternalLink target="_blank" href="https://www.equalto.com/tos">
            Terms of Service
          </ExternalLink>
          {' and '}
          <ExternalLink target="_blank" href="https://www.equalto.com/privacy-policy">
            Privacy Policy
          </ExternalLink>
          .
        </FormFooterText>
      </RightSide>
    </DualBox>
  );
};

function getSSOLoginURL(idp: string): string {
  const origin = window.location.hostname === 'localhost' ? 'http://localhost:5000' : 'https://sheets.equalto.com';
  return `${origin}/sso/${idp}/login`;
}

const Form = styled.form`
  display: grid;
  gap: 15px;
  width: 100%;
`;

const SSO = styled.div`
  display: grid;
  gap: 8px;
  width: 100%;
`;

export const SSOIcon = styled.div`
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 10px;
  svg {
    width: 18px;
    height: 18px;
  }
`;

const FormFooterText = styled.div`
  max-width: 180px;
  text-align: center;
  font-weight: 400;
  font-size: 9px;
  line-height: 11px;
  color: #8b8fad;
`;

const ErrorMessage = styled.div`
  color: #e06276;
  text-align: center;
`;

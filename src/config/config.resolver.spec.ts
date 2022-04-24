import { Test, TestingModule } from '@nestjs/testing';
import { ConfigResolver } from './config.resolver';
import { ConfigService } from './config.service';

describe('ConfigResolver', () => {
  let resolver: ConfigResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ConfigResolver, ConfigService],
    }).compile();

    resolver = module.get<ConfigResolver>(ConfigResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});

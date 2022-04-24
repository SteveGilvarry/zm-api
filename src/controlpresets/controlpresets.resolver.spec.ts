import { Test, TestingModule } from '@nestjs/testing';
import { ControlpresetsResolver } from './controlpresets.resolver';
import { ControlpresetsService } from './controlpresets.service';

describe('ControlpresetsResolver', () => {
  let resolver: ControlpresetsResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ControlpresetsResolver, ControlpresetsService],
    }).compile();

    resolver = module.get<ControlpresetsResolver>(ControlpresetsResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});

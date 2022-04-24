import { Test, TestingModule } from '@nestjs/testing';
import { ZonepresetsResolver } from './zonepresets.resolver';
import { ZonepresetsService } from './zonepresets.service';

describe('ZonepresetsResolver', () => {
  let resolver: ZonepresetsResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ZonepresetsResolver, ZonepresetsService],
    }).compile();

    resolver = module.get<ZonepresetsResolver>(ZonepresetsResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});

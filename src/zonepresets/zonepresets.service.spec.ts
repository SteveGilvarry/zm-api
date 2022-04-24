import { Test, TestingModule } from '@nestjs/testing';
import { ZonepresetsService } from './zonepresets.service';

describe('ZonepresetsService', () => {
  let service: ZonepresetsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ZonepresetsService],
    }).compile();

    service = module.get<ZonepresetsService>(ZonepresetsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});

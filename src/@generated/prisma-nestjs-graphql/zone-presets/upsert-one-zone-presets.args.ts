import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';
import { Type } from 'class-transformer';
import { ZonePresetsCreateInput } from './zone-presets-create.input';
import { ZonePresetsUpdateInput } from './zone-presets-update.input';

@ArgsType()
export class UpsertOneZonePresetsArgs {

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:false})
    @Type(() => ZonePresetsWhereUniqueInput)
    where!: ZonePresetsWhereUniqueInput;

    @Field(() => ZonePresetsCreateInput, {nullable:false})
    @Type(() => ZonePresetsCreateInput)
    create!: ZonePresetsCreateInput;

    @Field(() => ZonePresetsUpdateInput, {nullable:false})
    @Type(() => ZonePresetsUpdateInput)
    update!: ZonePresetsUpdateInput;
}

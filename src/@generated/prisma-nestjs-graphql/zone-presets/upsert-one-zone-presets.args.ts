import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';
import { ZonePresetsCreateInput } from './zone-presets-create.input';
import { ZonePresetsUpdateInput } from './zone-presets-update.input';

@ArgsType()
export class UpsertOneZonePresetsArgs {

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:false})
    where!: ZonePresetsWhereUniqueInput;

    @Field(() => ZonePresetsCreateInput, {nullable:false})
    create!: ZonePresetsCreateInput;

    @Field(() => ZonePresetsUpdateInput, {nullable:false})
    update!: ZonePresetsUpdateInput;
}

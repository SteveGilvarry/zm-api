import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';

@ArgsType()
export class DeleteOneZonePresetsArgs {

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:false})
    where!: ZonePresetsWhereUniqueInput;
}

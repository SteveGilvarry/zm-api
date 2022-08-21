import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneControlPresetsArgs {

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:false})
    @Type(() => ControlPresetsWhereUniqueInput)
    where!: ControlPresetsWhereUniqueInput;
}

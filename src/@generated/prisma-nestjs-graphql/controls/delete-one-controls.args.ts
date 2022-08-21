import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneControlsArgs {

    @Field(() => ControlsWhereUniqueInput, {nullable:false})
    @Type(() => ControlsWhereUniqueInput)
    where!: ControlsWhereUniqueInput;
}

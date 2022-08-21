import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsCreateInput } from './controls-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneControlsArgs {

    @Field(() => ControlsCreateInput, {nullable:false})
    @Type(() => ControlsCreateInput)
    data!: ControlsCreateInput;
}

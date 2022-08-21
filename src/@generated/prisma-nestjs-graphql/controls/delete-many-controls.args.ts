import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereInput } from './controls-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyControlsArgs {

    @Field(() => ControlsWhereInput, {nullable:true})
    @Type(() => ControlsWhereInput)
    where?: ControlsWhereInput;
}
